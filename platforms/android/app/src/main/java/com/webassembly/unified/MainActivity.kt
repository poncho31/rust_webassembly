package com.webassembly.unified

import android.os.Bundle
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.JavascriptInterface
import android.app.Activity
import android.util.Log
import android.webkit.WebSettings
import android.content.Intent
import android.provider.MediaStore
import android.graphics.Bitmap
import android.content.pm.PackageManager
import android.Manifest
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import android.widget.Toast

class MainActivity : Activity() {
    
    private lateinit var webView: WebView
    private val CAMERA_REQUEST = 1888
    private val CAMERA_PERMISSION_CODE = 100
    
    companion object {
        private const val TAG = "WebAssemblyApp"
        
        init {
            System.loadLibrary("webassembly_android")
        }
    }
    
    // Native methods
    private external fun initRust(): Boolean
    private external fun getServerUrl(): String
    private external fun handleWebViewMessage(message: String): String
    private external fun startEmbeddedServer(port: Int): Boolean    
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        Log.i(TAG, "=== APP STARTING ===")
        Log.i(TAG, "Android Version: ${android.os.Build.VERSION.SDK_INT}")
        Log.i(TAG, "App Package: ${packageName}")
        
        // Initialize Rust backend
        try {
            Log.i(TAG, "Attempting to initialize Rust backend...")
            if (initRust()) {
                Log.i(TAG, "✅ Rust backend initialized successfully")
                
                // Start embedded server before loading WebView
                Log.i(TAG, "Attempting to start embedded server on port 8088...")
                if (startEmbeddedServer(8080)) {
                    Log.i(TAG, "✅ Embedded server started successfully on port 8088")
                    
                    // Create and configure WebView after server is ready
                    webView = WebView(this)
                    setupWebView()
                    setContentView(webView)
                    
                    // Load HTML after server is running
                    loadLocalHtml()
                } else {
                    Log.e(TAG, "❌ Failed to start embedded server")
                    showError("Failed to start embedded server")
                }
            } else {
                Log.e(TAG, "❌ Failed to initialize Rust backend")
                showError("Failed to initialize Rust backend")
            }
        } catch (e: Exception) {
            Log.e(TAG, "❌ Exception during initialization: ${e.message}", e)
            showError("Initialization error: ${e.message}")
        }
    }
    
    private fun showError(message: String) {
        Log.e(TAG, "Showing error: $message")
        // Create a simple error display
        webView = WebView(this)
        setContentView(webView)
        val errorHtml = """
            <html><body style="font-family: Arial; padding: 20px; background: #ffebee;">
            <h1 style="color: #d32f2f;">❌ Error</h1>
            <p><strong>$message</strong></p>
            <p>Check Android logs for details:</p>
            <pre>adb logcat | grep WebAssemblyApp</pre>
            </body></html>
        """.trimIndent()
        webView.loadData(errorHtml, "text/html", "UTF-8")
    }
    
    private fun setupWebView() {
        webView.webViewClient = WebViewClient()
        val settings: WebSettings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.allowFileAccess = true
        settings.allowContentAccess = true
        settings.setAllowFileAccessFromFileURLs(true)
        settings.setAllowUniversalAccessFromFileURLs(true)
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
        settings.loadsImagesAutomatically = true
        settings.blockNetworkImage = false
        settings.blockNetworkLoads = false
        
        // Add JavaScript interface to communicate with Rust
        webView.addJavascriptInterface(WebAppInterface(), "Android")
    }    
    
    private fun loadLocalHtml() {
        try {
            val inputStream = assets.open("static/index.html")
            val htmlContent = inputStream.bufferedReader().use { it.readText() }
            
            webView.loadDataWithBaseURL("file:///android_asset/static/", htmlContent, "text/html", "UTF-8", null)
            Log.i(TAG, "Loading index.html and static assets from assets")
        } catch (e: Exception) {
            Log.e(TAG, "Error loading index.html: ${e.message}")
            // Fallback to direct URL loading
            webView.loadUrl("file:///android_asset/static/index.html")
        }
    }
    
    private fun checkCameraPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun requestCameraPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CAMERA), CAMERA_PERMISSION_CODE)
    }
    
    private fun openCamera() {
        val cameraIntent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
        if (cameraIntent.resolveActivity(packageManager) != null) {
            startActivityForResult(cameraIntent, CAMERA_REQUEST)
        } else {
            Toast.makeText(this, "Aucune application caméra trouvée", Toast.LENGTH_SHORT).show()
        }
    }
    
    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        when (requestCode) {
            CAMERA_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    openCamera()
                } else {
                    Toast.makeText(this, "Permission caméra refusée", Toast.LENGTH_SHORT).show()
                }
            }
        }
    }
    
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)

        if (requestCode == CAMERA_REQUEST && resultCode == RESULT_OK) {
            val photo = data?.extras?.get("data") as? Bitmap
            if (photo != null) {
                Log.i(TAG, "Photo prise avec succès")
                // Notifier JavaScript qu'une photo a été prise
                webView.evaluateJavascript(
                    "if(window.onPhotoTaken) window.onPhotoTaken('Photo prise avec succès');",
                    null
                )
                Toast.makeText(this, "Photo prise !", Toast.LENGTH_SHORT).show()
            }
        }
        
    }
    
    inner class WebAppInterface {
        @JavascriptInterface
        fun sendMessage(message: String): String {
            Log.d(TAG, "Received message from JavaScript: $message")
            return handleWebViewMessage(message)
        }
        
        @JavascriptInterface
        fun log(message: String) {
            Log.d(TAG, "JS Log: $message")
        }
        
        @JavascriptInterface
        fun takePhoto() {
            Log.d(TAG, "Camera button clicked from JavaScript")
            runOnUiThread {
                if (checkCameraPermission()) {
                    openCamera()
                } else {
                    requestCameraPermission()
                }
            }
        }
    }
    
    override fun onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
    }
}
