package com.main

import android.os.Bundle
import android.util.Log
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.ConsoleMessage
import android.webkit.PermissionRequest
import android.widget.RelativeLayout
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import android.os.Build
import android.content.Intent
import android.app.Activity
import com.main.permissions.PermissionManager
import com.main.media.AudioRecorder
import com.main.media.VideoRecorder
import com.main.media.CameraHandler
import com.main.handlers.ActivityResultHandler
import com.main.interfaces.WebAppInterface
import com.main.R

class MainActivity : AppCompatActivity() {
    
    // Native functions
    external fun initRust(): Boolean
    external fun getServerUrl(): String
    external fun testServerConnectivity(): Boolean
    
    private lateinit var webView: WebView
    private var hiddenSurfaceView: SurfaceView? = null
    private var surfaceHolder: SurfaceHolder? = null
    
    // Variables pour sauvegarder l'état
    private var webViewUrl: String? = null
    
    // Gestionnaires
    private lateinit var permissionManager: PermissionManager
    private lateinit var audioRecorder: AudioRecorder
    private lateinit var videoRecorder: VideoRecorder
    private lateinit var cameraHandler: CameraHandler
    private lateinit var activityResultHandler: ActivityResultHandler
    private lateinit var webAppInterface: WebAppInterface

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("rust_webassembly_android", "MainActivity onCreate started")
        
        // Initialiser les gestionnaires
        initializeManagers()
        
        // Restaurer l'état sauvegardé
        restoreState(savedInstanceState)
        
        // Keep screen on
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        
        setContentView(R.layout.activity_main)
        setupWebView()
        setupHiddenSurfaceView()
        setupBackPressedHandler()
        
        Log.d("rust_webassembly_android", "MainActivity onCreate completed")
    }
    
    private fun initializeManagers() {
        // Initialiser le gestionnaire de permissions avec callback
        permissionManager = PermissionManager(this) { pendingAction, isGranted ->
            handlePermissionResult(pendingAction, isGranted)
        }
        
        // Initialiser les gestionnaires média
        audioRecorder = AudioRecorder(this)
        videoRecorder = VideoRecorder(this)
        cameraHandler = CameraHandler(this)
        activityResultHandler = ActivityResultHandler(this)
        
        // Initialiser l'interface WebApp (sera fait après setupHiddenSurfaceView)
    }
    
    private fun handlePermissionResult(pendingAction: String?, isGranted: Boolean) {
        if (isGranted) {
            Toast.makeText(this, "Permission granted", Toast.LENGTH_SHORT).show()
            Log.d("rust_webassembly_android", "Permission granted - executing pending action: $pendingAction")
            
            // Exécuter l'action en attente
            when (pendingAction) {
                "takePhoto" -> {
                    permissionManager.clearPendingActions()
                    webView.post {
                        webAppInterface.executeCamera()
                    }
                }
                "recordVideo" -> {
                    permissionManager.clearPendingActions()
                    webView.post {
                        webAppInterface.executeVideoRecording()
                    }
                }
                "startRecording" -> {
                    permissionManager.clearPendingActions()
                    webView.post {
                        webAppInterface.executeAudioRecording()
                    }
                }
                "recordVideoBackground" -> {
                    permissionManager.clearPendingActions()
                    webView.post {
                        webAppInterface.executeVideoBackgroundRecording()
                    }
                }
                "sendSMS" -> {
                    permissionManager.clearPendingActions()
                    if (permissionManager.pendingSmsNumber != null && permissionManager.pendingSmsMessage != null) {
                        webView.post {
                            webAppInterface.executeSendSMS(permissionManager.pendingSmsNumber!!, permissionManager.pendingSmsMessage!!)
                        }
                        permissionManager.clearPendingActions()
                    }
                }
                "getLocation" -> {
                    permissionManager.clearPendingActions()
                    webView.post {
                        val location = webAppInterface.executeGetLocation()
                        webView.evaluateJavascript("window.handleLocationResult('$location');", null)
                    }
                }
            }
        } else {
            Toast.makeText(this, "Permission denied", Toast.LENGTH_SHORT).show()
            Log.d("rust_webassembly_android", "Permission denied")
            permissionManager.clearPendingActions()
        }
    }
    
    private fun restoreState(savedInstanceState: Bundle?) {
        savedInstanceState?.let { bundle ->
            val currentPhotoPath = bundle.getString("currentPhotoPath")
            val savedUrl = bundle.getString("webViewUrl")
            permissionManager.pendingAction = bundle.getString("pendingAction")
            permissionManager.pendingSmsNumber = bundle.getString("pendingSmsNumber")
            permissionManager.pendingSmsMessage = bundle.getString("pendingSmsMessage")
            
            Log.d("rust_webassembly_android", "Restored state - currentPhotoPath: $currentPhotoPath")
            Log.d("rust_webassembly_android", "Restored state - savedUrl: $savedUrl (ignoring for fresh server load)")
            
            // Don't restore webViewUrl - we want to always load from the server
            // webViewUrl = savedUrl
            
            // Restaurer l'état du gestionnaire de caméra
            cameraHandler.restoreState(currentPhotoPath)
        }
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        
        // Sauvegarder l'état
        outState.putString("currentPhotoPath", cameraHandler.saveState())
        outState.putString("webViewUrl", webView.url)
        outState.putString("pendingAction", permissionManager.pendingAction)
        outState.putString("pendingSmsNumber", permissionManager.pendingSmsNumber)
        outState.putString("pendingSmsMessage", permissionManager.pendingSmsMessage)
        
        Log.d("rust_webassembly_android", "State saved")
    }

    private fun setupWebView() {
        Log.d("rust_webassembly_android", "Setting up WebView")
        webView = findViewById(R.id.webview)
        
        // Enable JavaScript and other settings
        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            allowFileAccess = true
            allowContentAccess = true
            mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            databaseEnabled = true
            cacheMode = WebSettings.LOAD_DEFAULT
            
            // Allow network access for localhost
            allowFileAccessFromFileURLs = false // Disable for security
            allowUniversalAccessFromFileURLs = false // Disable for security
            blockNetworkImage = false
            blockNetworkLoads = false
            
            // Modern alternatives to deprecated methods
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.JELLY_BEAN) {
                // Keep these disabled when loading from HTTP server
                allowFileAccessFromFileURLs = false
                allowUniversalAccessFromFileURLs = false
            }
            
            // Enable debugging for WebView
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
                WebView.setWebContentsDebuggingEnabled(true)
            }
        }

        // L'interface WebApp sera ajoutée après la configuration de la surface
        
        // Set WebView client
        webView.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                Log.d("rust_webassembly_android", "Loading URL: ${request?.url}")
                return false
            }
            
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                Log.d("rust_webassembly_android", "Page finished loading: $url")
            }
            
            override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: WebResourceError?) {
                super.onReceivedError(view, request, error)
                Log.e("rust_webassembly_android", "WebView error: ${error?.description}")
            }
        }

        // Set WebChromeClient for console logs and other features
        webView.webChromeClient = object : WebChromeClient() {
            override fun onConsoleMessage(consoleMessage: ConsoleMessage?): Boolean {
                Log.d("rust_webassembly_android", "Console: ${consoleMessage?.message()} at ${consoleMessage?.sourceId()}:${consoleMessage?.lineNumber()}")
                return true
            }
            
            override fun onPermissionRequest(request: PermissionRequest?) {
                Log.d("rust_webassembly_android", "Permission request: ${request?.resources?.joinToString()}")
                request?.grant(request.resources)
            }
        }
    }

    private fun setupHiddenSurfaceView() {
        Log.d("rust_webassembly_android", "Setting up hidden surface view for camera preview")
        
        // Créer une SurfaceView cachée pour la prévisualisation de la caméra
        hiddenSurfaceView = SurfaceView(this).apply {
            layoutParams = RelativeLayout.LayoutParams(1, 1).apply {
                addRule(RelativeLayout.ALIGN_PARENT_TOP)
                addRule(RelativeLayout.ALIGN_PARENT_START)
            }
            alpha = 0.01f // Presque transparente
        }
        
        // Obtenir le SurfaceHolder
        surfaceHolder = hiddenSurfaceView!!.holder.apply {
            addCallback(object : SurfaceHolder.Callback {
                override fun surfaceCreated(holder: SurfaceHolder) {
                    Log.d("rust_webassembly_android", "Hidden surface created")
                }
                
                override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                    Log.d("rust_webassembly_android", "Hidden surface changed: ${width}x${height}")
                }
                
                override fun surfaceDestroyed(holder: SurfaceHolder) {
                    Log.d("rust_webassembly_android", "Hidden surface destroyed")
                }
            })
        }
        
        // Ajouter la SurfaceView cachée au layout principal (RelativeLayout)
        val rootLayout = webView.parent as? android.widget.RelativeLayout
        if (rootLayout != null) {
            rootLayout.addView(hiddenSurfaceView)
            Log.d("rust_webassembly_android", "Hidden surface view added to main layout")
        } else {
            Log.w("rust_webassembly_android", "Could not find RelativeLayout parent for webview")
        }
        
        // Maintenant initialiser l'interface WebApp avec la surface
        webAppInterface = WebAppInterface(
            this,
            permissionManager,
            audioRecorder,
            videoRecorder,
            cameraHandler,
            surfaceHolder
        )
        
        // Add JavaScript interface
        webView.addJavascriptInterface(webAppInterface, "Android")
        
        // Load the main page
        loadMainPage()
    }

    private fun setupBackPressedHandler() {
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (webView.canGoBack()) {
                    webView.goBack()
                } else {
                    finish()
                }
            }
        })
    }

    private fun loadMainPage() {
        Log.d("rust_webassembly_android", "loadMainPage() called")
        
        // Initialize Rust backend and start the embedded server
        Log.d("rust_webassembly_android", "Initializing Rust backend...")
        val serverStarted = initRust()
        Log.d("rust_webassembly_android", "Rust initialization result: $serverStarted")
        
        if (!serverStarted) {
            Log.e("rust_webassembly_android", "Failed to start Rust server")
            // Fallback to asset files if server fails to start
            val assetPath = "file:///android_asset/static/index.html"
            Log.d("rust_webassembly_android", "Falling back to asset path: $assetPath")
            webView.loadUrl(assetPath)
            return
        }
        
        // Test server connectivity after startup
        Log.d("rust_webassembly_android", "Testing server connectivity...")
        val isConnectable = testServerConnectivity()
        Log.d("rust_webassembly_android", "Server connectivity test result: $isConnectable")
        
        if (!isConnectable) {
            Log.w("rust_webassembly_android", "Server is not connectable yet, waiting and retrying...")
            
            // Try again after a short delay
            android.os.Handler(android.os.Looper.getMainLooper()).postDelayed({
                val retryConnectivity = testServerConnectivity()
                Log.d("rust_webassembly_android", "Retry connectivity test result: $retryConnectivity")
                
                if (!retryConnectivity) {
                    Log.e("rust_webassembly_android", "Server still not connectable after retry, falling back to assets")
                    val assetPath = "file:///android_asset/static/index.html"
                    Log.d("rust_webassembly_android", "Falling back to asset path: $assetPath")
                    webView.loadUrl(assetPath)
                    return@postDelayed
                }
                
                // Get the server URL from the Rust backend
                Log.d("rust_webassembly_android", "Getting server URL from Rust...")
                val serverUrl = getServerUrl()
                Log.d("rust_webassembly_android", "Server URL received: $serverUrl")
                Log.d("rust_webassembly_android", "Loading WebView with server URL: $serverUrl")
                webView.loadUrl(serverUrl)
                Log.d("rust_webassembly_android", "WebView.loadUrl() called with: $serverUrl")
            }, 2000) // Wait 2 seconds before retry
            
            return
        }
        
        // Get the server URL from the Rust backend
        Log.d("rust_webassembly_android", "Getting server URL from Rust...")
        val serverUrl = getServerUrl()
        Log.d("rust_webassembly_android", "Server URL received: $serverUrl")
        Log.d("rust_webassembly_android", "Loading WebView with server URL: $serverUrl")
        webView.loadUrl(serverUrl)
        Log.d("rust_webassembly_android", "WebView.loadUrl() called with: $serverUrl")
    }

    @Deprecated("This method has been deprecated in favor of using the Activity Result API")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        activityResultHandler.handleActivityResult(requestCode, resultCode, data, cameraHandler)
    }
    
    companion object {
        init {
            System.loadLibrary("webassembly_android")
        }
    }
}
