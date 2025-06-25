package com.webassembly.unified

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.hardware.Camera
import android.location.LocationManager
import android.media.MediaRecorder
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.provider.MediaStore
import android.telephony.SmsManager
import android.util.Log
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import android.webkit.*
import android.widget.FrameLayout
import android.widget.RelativeLayout
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.*
import java.util.Date

class MainActivity : AppCompatActivity() {
    
    private lateinit var webView: WebView
    private var mediaRecorder: MediaRecorder? = null
    private var videoMediaRecorder: MediaRecorder? = null
    private var camera: Camera? = null
    private var previewLayout: FrameLayout? = null
    private var hiddenSurfaceView: SurfaceView? = null
    private var surfaceHolder: SurfaceHolder? = null
    private var isRecording = false
    private var isVideoRecording = false
    private var outputFile: File? = null
    private var videoOutputFile: File? = null
    private var photoFile: File? = null
    
    // Variables pour mémoriser les actions en attente de permission
    private var pendingAction: String? = null
    private var pendingSmsNumber: String? = null
    private var pendingSmsMessage: String? = null
    
    // Variables pour sauvegarder l'état
    private var currentPhotoPath: String? = null
    private var webViewUrl: String? = null
    
    // Request codes
    private val CAMERA_REQUEST_CODE = 100
    private val STORAGE_REQUEST_CODE = 101
    private val LOCATION_REQUEST_CODE = 102
    private val MICROPHONE_REQUEST_CODE = 103
    private val SMS_REQUEST_CODE = 104
    private val PICK_IMAGE_REQUEST = 1
    private val CAMERA_CAPTURE_REQUEST = 2
    
    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted: Boolean ->
        if (isGranted) {
            Toast.makeText(this, "Permission granted", Toast.LENGTH_SHORT).show()
            Log.d("WebAssemblyApp", "Permission granted - executing pending action: $pendingAction")
            
            // Exécuter l'action en attente
            when (pendingAction) {
                "takePhoto" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeCamera()
                            ?: WebAppInterface(this).executeCamera()
                    }
                }
                "recordVideo" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeVideoRecording()
                            ?: WebAppInterface(this).executeVideoRecording()
                    }                }                "startRecording" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeAudioRecording()
                            ?: WebAppInterface(this).executeAudioRecording()
                    }
                }
                "recordVideoBackground" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeVideoBackgroundRecording()
                            ?: WebAppInterface(this).executeVideoBackgroundRecording()
                    }
                }
                "sendSMS" -> {
                    pendingAction = null
                    if (pendingSmsNumber != null && pendingSmsMessage != null) {
                        findViewById<WebView>(R.id.webview).post {
                            (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeSendSMS(pendingSmsNumber!!, pendingSmsMessage!!)
                                ?: WebAppInterface(this).executeSendSMS(pendingSmsNumber!!, pendingSmsMessage!!)
                        }
                        pendingSmsNumber = null
                        pendingSmsMessage = null
                    }
                }
                "getLocation" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        val location = (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeGetLocation()
                            ?: WebAppInterface(this).executeGetLocation()
                        webView.evaluateJavascript("window.handleLocationResult('$location');", null)
                    }
                }
            }
        } else {
            Toast.makeText(this, "Permission denied", Toast.LENGTH_SHORT).show()
            Log.d("WebAssemblyApp", "Permission denied")
            pendingAction = null
            pendingSmsNumber = null
            pendingSmsMessage = null
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("WebAssemblyApp", "MainActivity onCreate started")
        
        // Restaurer l'état sauvegardé
        savedInstanceState?.let { bundle ->
            currentPhotoPath = bundle.getString("currentPhotoPath")
            webViewUrl = bundle.getString("webViewUrl")
            pendingAction = bundle.getString("pendingAction")
            pendingSmsNumber = bundle.getString("pendingSmsNumber")
            pendingSmsMessage = bundle.getString("pendingSmsMessage")
            
            Log.d("WebAssemblyApp", "Restored state - currentPhotoPath: $currentPhotoPath")
            
            // Restaurer le fichier photo si il existe
            currentPhotoPath?.let { path ->
                photoFile = File(path)
                if (!photoFile!!.exists()) {
                    photoFile = null
                    currentPhotoPath = null
                }
            }
        }
        
        // Keep screen on
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        
        setContentView(R.layout.activity_main)
          setupWebView()
        setupHiddenSurfaceView()
        setupBackPressedHandler()
        
        Log.d("WebAssemblyApp", "MainActivity onCreate completed")
    }

    private fun setupWebView() {
        Log.d("WebAssemblyApp", "Setting up WebView")
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
              // Modern alternatives to deprecated methods
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.JELLY_BEAN) {
                allowFileAccessFromFileURLs = true
                allowUniversalAccessFromFileURLs = true
            }
            
            // Enable debugging for WebView
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
                WebView.setWebContentsDebuggingEnabled(true)
            }
        }

        // Add JavaScript interface
        webView.addJavascriptInterface(WebAppInterface(this), "Android")
        
        // Set WebView client
        webView.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                Log.d("WebAssemblyApp", "Loading URL: ${request?.url}")
                return false
            }
            
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                Log.d("WebAssemblyApp", "Page finished loading: $url")
            }
            
            override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: WebResourceError?) {
                super.onReceivedError(view, request, error)
                Log.e("WebAssemblyApp", "WebView error: ${error?.description}")
            }
        }

        // Set WebChromeClient for console logs and other features
        webView.webChromeClient = object : WebChromeClient() {
            override fun onConsoleMessage(consoleMessage: ConsoleMessage?): Boolean {
                Log.d("WebAssemblyApp", "Console: ${consoleMessage?.message()} at ${consoleMessage?.sourceId()}:${consoleMessage?.lineNumber()}")
                return true
            }
            
            override fun onPermissionRequest(request: PermissionRequest?) {
                Log.d("WebAssemblyApp", "Permission request: ${request?.resources?.joinToString()}")
                request?.grant(request.resources)
            }
        }

        // Load the main page
        loadMainPage()
    }

    private fun setupHiddenSurfaceView() {
        Log.d("WebAssemblyApp", "Setting up hidden surface view for camera preview")
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
                    Log.d("WebAssemblyApp", "Hidden surface created")
                }
                
                override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                    Log.d("WebAssemblyApp", "Hidden surface changed: ${width}x${height}")
                }
                
                override fun surfaceDestroyed(holder: SurfaceHolder) {
                    Log.d("WebAssemblyApp", "Hidden surface destroyed")
                }
            })
        }
        
        // Ajouter la SurfaceView cachée au layout principal (RelativeLayout)
        val rootLayout = webView.parent as? android.widget.RelativeLayout
        if (rootLayout != null) {
            rootLayout.addView(hiddenSurfaceView)
            Log.d("WebAssemblyApp", "Hidden surface view added to main layout")
        } else {
            Log.w("WebAssemblyApp", "Could not find RelativeLayout parent for webview")
        }
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
        val assetPath = "file:///android_asset/static/index.html"
        Log.d("WebAssemblyApp", "Loading main page: $assetPath")
        webView.loadUrl(assetPath)
    }

    inner class WebAppInterface(private val context: Context) {

        @JavascriptInterface
        fun log(message: String) {
            Log.d("WebAssemblyApp", "JS Log: $message")
        }

        @JavascriptInterface
        fun error(message: String) {
            Log.e("WebAssemblyApp", "JS Error: $message")
        }

        @JavascriptInterface
        fun warn(message: String) {
            Log.w("WebAssemblyApp", "JS Warning: $message")
        }

        @JavascriptInterface
        fun info(message: String) {
            Log.i("WebAssemblyApp", "JS Info: $message")
        }        @JavascriptInterface
        fun requestPermission(permission: String) {
            Log.d("WebAssemblyApp", "Requesting permission: $permission")
            when (permission) {
                "camera" -> requestPermissionLauncher.launch(Manifest.permission.CAMERA)
                "microphone" -> requestPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                "location" -> requestPermissionLauncher.launch(Manifest.permission.ACCESS_FINE_LOCATION)
                "sms" -> requestPermissionLauncher.launch(Manifest.permission.SEND_SMS)
                "storage" -> {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        requestPermissionLauncher.launch(Manifest.permission.READ_MEDIA_IMAGES)
                    } else {
                        requestPermissionLauncher.launch(Manifest.permission.WRITE_EXTERNAL_STORAGE)
                    }
                }
            }
        }        
        @JavascriptInterface
        fun checkPermission(permission: String): Boolean {
            val result = when (permission) {
                "camera" -> ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
                "microphone" -> ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
                "location" -> ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED
                "sms" -> ContextCompat.checkSelfPermission(context, Manifest.permission.SEND_SMS) == PackageManager.PERMISSION_GRANTED
                "storage" -> {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        ContextCompat.checkSelfPermission(context, Manifest.permission.READ_MEDIA_IMAGES) == PackageManager.PERMISSION_GRANTED
                    } else {                        ContextCompat.checkSelfPermission(context, Manifest.permission.WRITE_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
                    }
                }
                else -> false
            }
            
            Log.d("WebAssemblyApp", "Permission $permission: $result")
            return result
        }

        @JavascriptInterface
        fun startRecording() {
            Log.d("WebAssemblyApp", "Toggle audio recording - currently recording: ${this@MainActivity.isRecording}")
            
            if (this@MainActivity.isRecording) {
                // Si on est en train d'enregistrer, arrêter l'enregistrement
                stopRecordingInternal()
            } else {
                // Si on n'enregistre pas, démarrer l'enregistrement
                if (ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) 
                    == PackageManager.PERMISSION_GRANTED) {
                    executeAudioRecording()
                } else {
                    Log.d("WebAssemblyApp", "Requesting microphone permission for recording")
                    pendingAction = "startRecording"
                    requestPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                }            }
        }        fun executeAudioRecording() {
            try {
                val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
                val fileName = "REC_${timeStamp}.3gp"
                
                // Pour Android 10+ (API 29+), utiliser le répertoire privé de l'app d'abord
                val audioDir = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    // Utiliser le répertoire privé de l'app pour l'enregistrement
                    getExternalFilesDir(Environment.DIRECTORY_MUSIC)
                } else {
                    // Pour les versions antérieures, utiliser le répertoire public
                    val musicDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MUSIC)
                    val appDir = File(musicDir, "WebAssemblyApp")
                    appDir.mkdirs()
                    appDir
                }
                
                outputFile = File(audioDir, fileName)
                
                mediaRecorder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    MediaRecorder(context)
                } else {
                    @Suppress("DEPRECATION")
                    MediaRecorder()
                }.apply {
                    setAudioSource(MediaRecorder.AudioSource.MIC)
                    setOutputFormat(MediaRecorder.OutputFormat.THREE_GPP)
                    setOutputFile(outputFile!!.absolutePath)
                    setAudioEncoder(MediaRecorder.AudioEncoder.AMR_NB)
                    prepare()
                    start()
                }
                this@MainActivity.isRecording = true
                Log.d("WebAssemblyApp", "Recording started: ${outputFile!!.absolutePath}")
                
                runOnUiThread {
                    Toast.makeText(context, "🎤 Enregistrement démarré", Toast.LENGTH_SHORT).show()
                }
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Recording failed: ${e.message}")
                runOnUiThread {
                    Toast.makeText(context, "Erreur d'enregistrement: ${e.message}", Toast.LENGTH_SHORT).show()
                }            }
        }        private fun stopRecordingInternal(): String {
            Log.d("WebAssemblyApp", "Stopping audio recording internally")
            return try {
                mediaRecorder?.apply {
                    stop()
                    release()
                }
                mediaRecorder = null
                this@MainActivity.isRecording = false
                val filePath = outputFile?.absolutePath ?: ""
                Log.d("WebAssemblyApp", "Recording stopped: $filePath")
                
                // Copier le fichier vers le répertoire public pour qu'il soit accessible dans l'app musique
                if (filePath.isNotEmpty()) {
                    val sourceFile = File(filePath)
                    if (sourceFile.exists()) {
                        try {
                            val publicFilePath = copyAudioToPublicDirectory(sourceFile)
                            if (publicFilePath != null) {
                                Log.d("WebAssemblyApp", "Audio copied to public directory: $publicFilePath")
                                runOnUiThread {
                                    Toast.makeText(context, "🎵 Enregistrement sauvé dans Musique: ${sourceFile.name} (${formatFileSize(sourceFile.length())})", Toast.LENGTH_LONG).show()
                                }
                                return publicFilePath
                            } else {
                                // Si la copie échoue, garder le fichier dans le répertoire privé
                                Log.w("WebAssemblyApp", "Failed to copy to public directory, keeping in private directory")
                                runOnUiThread {
                                    Toast.makeText(context, "🎵 Enregistrement sauvé: ${sourceFile.name} (${formatFileSize(sourceFile.length())})", Toast.LENGTH_LONG).show()
                                }
                            }
                        } catch (e: Exception) {
                            Log.e("WebAssemblyApp", "Error copying audio to public directory: ${e.message}")
                            // Garder le fichier dans le répertoire privé en cas d'erreur
                            runOnUiThread {
                                Toast.makeText(context, "🎵 Enregistrement sauvé: ${sourceFile.name}", Toast.LENGTH_SHORT).show()
                            }
                        }
                    }
                } else {
                    runOnUiThread {
                        Toast.makeText(context, "⏹️ Enregistrement arrêté", Toast.LENGTH_SHORT).show()
                    }
                }
                filePath
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Stop recording failed: ${e.message}")
                this@MainActivity.isRecording = false
                runOnUiThread {
                    Toast.makeText(context, "Erreur arrêt enregistrement: ${e.message}", Toast.LENGTH_SHORT).show()
                }
                ""            }
        }

        @JavascriptInterface
        fun stopRecording(): String {
            Log.d("WebAssemblyApp", "Stop recording called from JS")
            return stopRecordingInternal()
        }

        @JavascriptInterface
        fun recordAudio() {
            Log.d("WebAssemblyApp", "Record audio called - toggle mode")
            startRecording() // Utilise la même logique de toggle
        }        @JavascriptInterface
        fun isRecording(): Boolean {
            Log.d("WebAssemblyApp", "Checking recording status: ${this@MainActivity.isRecording}")
            return this@MainActivity.isRecording
        }        @JavascriptInterface
        fun recordVideoBackground() {
            Log.d("WebAssemblyApp", "Toggle video recording - currently recording: ${this@MainActivity.isVideoRecording}")
            
            if (this@MainActivity.isVideoRecording) {
                // Si on est en train d'enregistrer, arrêter l'enregistrement vidéo
                stopVideoRecordingInternal()
            } else {
                // Si on n'enregistre pas, démarrer l'enregistrement vidéo
                val hasCamera = ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
                val hasMicrophone = ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
                
                if (hasCamera && hasMicrophone) {
                    executeVideoBackgroundRecording()
                } else {
                    Log.d("WebAssemblyApp", "Missing permissions for video recording - Camera: $hasCamera, Microphone: $hasMicrophone")
                    pendingAction = "recordVideoBackground"
                    
                    // Demander d'abord la permission caméra si elle manque
                    if (!hasCamera) {
                        requestPermissionLauncher.launch(Manifest.permission.CAMERA)
                    } else if (!hasMicrophone) {
                        requestPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                    }
                }
            }
        }

        @JavascriptInterface
        fun isVideoRecording(): Boolean {
            Log.d("WebAssemblyApp", "Checking video recording status: ${this@MainActivity.isVideoRecording}")
            return this@MainActivity.isVideoRecording
        }        fun executeVideoBackgroundRecording() {
            try {
                // Vérifier les permissions d'abord
                val hasCamera = ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
                val hasMicrophone = ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
                
                if (!hasCamera || !hasMicrophone) {
                    Log.e("WebAssemblyApp", "Missing permissions - Camera: $hasCamera, Microphone: $hasMicrophone")
                    runOnUiThread {
                        Toast.makeText(context, "Permissions caméra et microphone requises", Toast.LENGTH_SHORT).show()
                    }
                    return
                }
                
                val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
                val fileName = "VID_${timeStamp}.mp4"
                
                // Pour Android 10+ (API 29+), utiliser le répertoire privé de l'app d'abord
                val videoDir = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    // Utiliser le répertoire privé de l'app pour l'enregistrement
                    getExternalFilesDir(Environment.DIRECTORY_MOVIES)
                } else {
                    // Pour les versions antérieures, utiliser le répertoire public
                    val moviesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MOVIES)
                    val appDir = File(moviesDir, "WebAssemblyApp")
                    appDir.mkdirs()
                    appDir
                }
                
                videoOutputFile = File(videoDir, fileName)
                  // Première étape: ouvrir la caméra et configurer la prévisualisation
                try {
                    camera = Camera.open()
                    
                    // Vérifier que la surface est disponible
                    if (surfaceHolder == null) {
                        Log.e("WebAssemblyApp", "Surface holder not available")
                        throw Exception("Surface holder not available")
                    }
                    
                    // Configurer les paramètres de la caméra
                    val parameters = camera!!.parameters
                    val supportedVideoSizes = parameters.supportedVideoSizes
                    if (supportedVideoSizes != null && supportedVideoSizes.isNotEmpty()) {
                        // Chercher une taille 720p ou similaire
                        val preferredSize = supportedVideoSizes.find { it.width == 1280 && it.height == 720 }
                            ?: supportedVideoSizes.find { it.width <= 1280 && it.height <= 720 }
                            ?: supportedVideoSizes[0]
                        
                        parameters.setPreviewSize(preferredSize.width, preferredSize.height)
                        Log.d("WebAssemblyApp", "Camera preview size set to: ${preferredSize.width}x${preferredSize.height}")
                    }
                    
                    camera!!.parameters = parameters
                    
                    // Configurer la prévisualisation avec la surface cachée
                    camera!!.setPreviewDisplay(surfaceHolder)
                    camera!!.startPreview()
                    Log.d("WebAssemblyApp", "Camera preview started")
                    
                    // Déverrouiller la caméra pour MediaRecorder
                    camera!!.unlock()
                } catch (e: Exception) {
                    Log.e("WebAssemblyApp", "Failed to open camera and start preview: ${e.message}")
                    camera?.release()
                    camera = null
                    runOnUiThread {
                        Toast.makeText(context, "Impossible d'ouvrir la caméra: ${e.message}", Toast.LENGTH_SHORT).show()
                    }
                    return
                }
                
                videoMediaRecorder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    MediaRecorder(context)
                } else {
                    @Suppress("DEPRECATION")
                    MediaRecorder()
                }.apply {
                    // Configuration pour enregistrement vidéo en arrière-plan
                    setCamera(camera) // Assigner la caméra au MediaRecorder
                    setAudioSource(MediaRecorder.AudioSource.MIC)
                    setVideoSource(MediaRecorder.VideoSource.CAMERA)
                    setOutputFormat(MediaRecorder.OutputFormat.MPEG_4)
                    setOutputFile(videoOutputFile!!.absolutePath)
                    setVideoEncoder(MediaRecorder.VideoEncoder.H264)
                    setAudioEncoder(MediaRecorder.AudioEncoder.AAC)
                    setVideoSize(1280, 720) // 720p
                    setVideoFrameRate(30)
                    setVideoEncodingBitRate(5000000)
                    setAudioEncodingBitRate(128000)
                    setAudioSamplingRate(44100)
                    
                    try {
                        prepare()
                        start()
                    } catch (e: Exception) {
                        Log.e("WebAssemblyApp", "MediaRecorder prepare/start failed: ${e.message}")
                        // Nettoyer en cas d'erreur
                        release()
                        camera?.release()
                        camera = null
                        throw e
                    }
                }
                this@MainActivity.isVideoRecording = true
                Log.d("WebAssemblyApp", "Video recording started: ${videoOutputFile!!.absolutePath}")
                
                runOnUiThread {
                    Toast.makeText(context, "🎥 Enregistrement vidéo démarré", Toast.LENGTH_SHORT).show()
                }
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
                // Nettoyer en cas d'erreur
                videoMediaRecorder?.release()
                videoMediaRecorder = null
                camera?.release()
                camera = null
                this@MainActivity.isVideoRecording = false
                
                runOnUiThread {
                    Toast.makeText(context, "Erreur enregistrement vidéo: ${e.message}", Toast.LENGTH_SHORT).show()
                }
            }
        }        private fun stopVideoRecordingInternal(): String {
            Log.d("WebAssemblyApp", "Stopping video recording internally")
            return try {
                videoMediaRecorder?.apply {
                    stop()
                    release()
                }
                videoMediaRecorder = null
                  // Libérer la caméra
                camera?.apply {
                    try {
                        stopPreview() // Arrêter la prévisualisation
                        lock() // Verrouiller à nouveau la caméra
                        release()
                        Log.d("WebAssemblyApp", "Camera preview stopped and camera released")
                    } catch (e: Exception) {
                        Log.w("WebAssemblyApp", "Error stopping camera preview: ${e.message}")
                        release() // S'assurer que la caméra est libérée même en cas d'erreur
                    }
                }
                camera = null
                
                this@MainActivity.isVideoRecording = false
                val filePath = videoOutputFile?.absolutePath ?: ""
                Log.d("WebAssemblyApp", "Video recording stopped: $filePath")
                
                // Copier le fichier vers le répertoire public pour qu'il soit accessible dans l'app galerie
                if (filePath.isNotEmpty()) {
                    val sourceFile = File(filePath)
                    if (sourceFile.exists()) {
                        try {
                            val publicFilePath = copyVideoToPublicDirectory(sourceFile)
                            if (publicFilePath != null) {
                                Log.d("WebAssemblyApp", "Video copied to public directory: $publicFilePath")
                                runOnUiThread {
                                    Toast.makeText(context, "🎬 Vidéo sauvée dans Galerie: ${sourceFile.name} (${formatFileSize(sourceFile.length())})", Toast.LENGTH_LONG).show()
                                }
                                return publicFilePath
                            } else {
                                // Si la copie échoue, garder le fichier dans le répertoire privé
                                Log.w("WebAssemblyApp", "Failed to copy video to public directory, keeping in private directory")
                                runOnUiThread {
                                    Toast.makeText(context, "🎬 Vidéo sauvée: ${sourceFile.name} (${formatFileSize(sourceFile.length())})", Toast.LENGTH_LONG).show()
                                }
                            }
                        } catch (e: Exception) {
                            Log.e("WebAssemblyApp", "Error copying video to public directory: ${e.message}")
                            // Garder le fichier dans le répertoire privé en cas d'erreur
                            runOnUiThread {
                                Toast.makeText(context, "🎬 Vidéo sauvée: ${sourceFile.name}", Toast.LENGTH_SHORT).show()
                            }
                        }
                    }
                } else {
                    runOnUiThread {
                        Toast.makeText(context, "⏹️ Enregistrement vidéo arrêté", Toast.LENGTH_SHORT).show()
                    }
                }
                filePath
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Stop video recording failed: ${e.message}")
                // Nettoyer en cas d'erreur
                videoMediaRecorder?.release()
                videoMediaRecorder = null
                camera?.release()
                camera = null
                this@MainActivity.isVideoRecording = false
                
                runOnUiThread {
                    Toast.makeText(context, "Erreur arrêt enregistrement vidéo: ${e.message}", Toast.LENGTH_SHORT).show()
                }
                ""
            }
        }

        @JavascriptInterface
        fun stopVideoRecording(): String {
            Log.d("WebAssemblyApp", "Stop video recording called from JS")
            return stopVideoRecordingInternal()
        }

        @JavascriptInterface
        fun vibrate(duration: Long) {
            Log.d("WebAssemblyApp", "Vibrating for ${duration}ms")
            val vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val vibratorManager = getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
                vibratorManager.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                getSystemService(Context.VIBRATOR_SERVICE) as Vibrator            }
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
            } else {
                @Suppress("DEPRECATION")
                vibrator.vibrate(duration)
            }
        }

        @JavascriptInterface
        fun sendSMS(number: String, message: String): Boolean {
            Log.d("WebAssemblyApp", "Sending SMS to $number: $message")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.SEND_SMS) 
                == PackageManager.PERMISSION_GRANTED) {
                
                return executeSendSMS(number, message)
            } else {
                Log.d("WebAssemblyApp", "Requesting SMS permission")
                pendingAction = "sendSMS"
                pendingSmsNumber = number
                pendingSmsMessage = message
                requestPermissionLauncher.launch(Manifest.permission.SEND_SMS)
                return false
            }
        }

        fun executeSendSMS(number: String, message: String): Boolean {
            return try {
                val smsManager = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    context.getSystemService(SmsManager::class.java)
                } else {
                    @Suppress("DEPRECATION")
                    SmsManager.getDefault()                }
                smsManager.sendTextMessage(number, null, message, null, null)
                Log.d("WebAssemblyApp", "SMS sent successfully")
                true
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "SMS failed: ${e.message}")
                false
            }
        }        
        
        @JavascriptInterface
        fun getLocation(): String {
            Log.d("WebAssemblyApp", "Getting location")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_FINE_LOCATION) 
                == PackageManager.PERMISSION_GRANTED) {
                
                val result = executeGetLocation()
                
                // Afficher le résultat dans un popup comme pour getDeviceInfo
                runOnUiThread {
                    try {
                        val locationJson = JSONObject(result)
                        if (locationJson.has("latitude") && locationJson.has("longitude")) {
                            val lat = locationJson.getDouble("latitude")
                            val lng = locationJson.getDouble("longitude")
                            Toast.makeText(context, "Position: $lat, $lng", Toast.LENGTH_LONG).show()
                        } else {
                            Toast.makeText(context, "Position non disponible", Toast.LENGTH_SHORT).show()
                        }
                    } catch (e: Exception) {
                        Toast.makeText(context, "Erreur de position: ${e.message}", Toast.LENGTH_SHORT).show()
                    }
                }
                
                return result
            } else {
                Log.d("WebAssemblyApp", "Requesting location permission")
                pendingAction = "getLocation"
                requestPermissionLauncher.launch(Manifest.permission.ACCESS_FINE_LOCATION)
                return "{\"error\": \"Permission not granted\"}"
            }
        }

        fun executeGetLocation(): String {
            val locationManager = getSystemService(Context.LOCATION_SERVICE) as LocationManager
            val location = locationManager.getLastKnownLocation(LocationManager.GPS_PROVIDER)
              return if (location != null) {
                val result = "{\"latitude\": ${location.latitude}, \"longitude\": ${location.longitude}}"
                Log.d("WebAssemblyApp", "Location: $result")
                result
            } else {
                Log.d("WebAssemblyApp", "Location not available")
                "{\"error\": \"Location not available\"}"
            }
        }

        @JavascriptInterface
        fun takePhoto() {
            Log.d("WebAssemblyApp", "Taking photo")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) 
                == PackageManager.PERMISSION_GRANTED) {
                executeCamera()
            } else {
                Log.d("WebAssemblyApp", "Requesting camera permission for photo")
                pendingAction = "takePhoto"
                requestPermissionLauncher.launch(Manifest.permission.CAMERA)
            }
        }

        @JavascriptInterface
        fun recordVideo() {
            Log.d("WebAssemblyApp", "Recording video")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) 
                == PackageManager.PERMISSION_GRANTED) {
                executeVideoRecording()
            } else {
                Log.d("WebAssemblyApp", "Requesting camera permission for video")
                pendingAction = "recordVideo"
                requestPermissionLauncher.launch(Manifest.permission.CAMERA)
            }
        }        // Méthodes d'exécution séparées
        fun executeCamera() {
            try {
                photoFile = createImageFile()
                currentPhotoPath = photoFile?.absolutePath // Sauvegarder le chemin
                
                val photoURI = FileProvider.getUriForFile(
                    this@MainActivity,
                    "com.webassembly.unified.fileprovider",
                    photoFile!!
                )
                
                Log.d("WebAssemblyApp", "Photo will be saved to: $currentPhotoPath")
                
                val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
                    putExtra(MediaStore.EXTRA_OUTPUT, photoURI)
                    // Ajouter des flags pour améliorer la compatibilité
                    addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                    addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                }
                
                // Vérifier qu'une app peut gérer cette intent
                if (intent.resolveActivity(packageManager) != null) {
                    startActivityForResult(intent, CAMERA_CAPTURE_REQUEST)
                } else {
                    Log.e("WebAssemblyApp", "No camera app available")
                    runOnUiThread {
                        Toast.makeText(context, "Aucune application appareil photo disponible", Toast.LENGTH_SHORT).show()
                    }
                }
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
                runOnUiThread {
                    Toast.makeText(context, "Erreur appareil photo: ${e.message}", Toast.LENGTH_SHORT).show()
                }
            }
        }

        fun executeVideoRecording() {
            try {
                val intent = Intent(MediaStore.ACTION_VIDEO_CAPTURE)
                startActivityForResult(intent, 3)
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
            }
        }        @JavascriptInterface
        fun openCamera() {
            Log.d("WebAssemblyApp", "Opening camera")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) 
                == PackageManager.PERMISSION_GRANTED) {
                
                try {
                    photoFile = createImageFile()
                    currentPhotoPath = photoFile?.absolutePath // Sauvegarder le chemin
                    
                    val photoURI = FileProvider.getUriForFile(
                        this@MainActivity,
                        "com.webassembly.unified.fileprovider",
                        photoFile!!
                    )
                    
                    Log.d("WebAssemblyApp", "Photo will be saved to: $currentPhotoPath")
                    
                    val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
                        putExtra(MediaStore.EXTRA_OUTPUT, photoURI)
                        addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    }
                    
                    if (intent.resolveActivity(packageManager) != null) {
                        startActivityForResult(intent, CAMERA_CAPTURE_REQUEST)
                    } else {
                        Log.e("WebAssemblyApp", "No camera app available")
                        runOnUiThread {
                            Toast.makeText(context, "Aucune application appareil photo disponible", Toast.LENGTH_SHORT).show()
                        }
                    }
                } catch (e: Exception) {
                    Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
                    runOnUiThread {
                        Toast.makeText(context, "Erreur appareil photo: ${e.message}", Toast.LENGTH_SHORT).show()
                    }
                }
            } else {
                Log.e("WebAssemblyApp", "No permission for camera")
                runOnUiThread {
                    Toast.makeText(context, "Permission appareil photo requise", Toast.LENGTH_SHORT).show()
                }
            }
        }

        @JavascriptInterface
        fun openGallery() {
            Log.d("WebAssemblyApp", "Opening gallery")
            val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
                type = "image/*"
                addCategory(Intent.CATEGORY_OPENABLE)
            }
            startActivityForResult(intent, PICK_IMAGE_REQUEST)
        }

        @JavascriptInterface
        fun openFile() {
            Log.d("WebAssemblyApp", "Opening file picker")
            val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
                type = "*/*"
                addCategory(Intent.CATEGORY_OPENABLE)
            }
            startActivityForResult(intent, 1001)
        }

        @JavascriptInterface
        fun saveFile(filename: String, content: String): Boolean {
            Log.d("WebAssemblyApp", "Saving file: $filename")
            return try {
                val file = File(context.externalCacheDir, filename)
                FileOutputStream(file).use { 
                    it.write(content.toByteArray()) 
                }
                Log.d("WebAssemblyApp", "File saved: ${file.absolutePath}")
                true
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Save file failed: ${e.message}")
                false
            }
        }

        @JavascriptInterface
        fun getDeviceInfo(): String {
            val info = JSONObject().apply {
                put("model", Build.MODEL)
                put("manufacturer", Build.MANUFACTURER)
                put("version", Build.VERSION.RELEASE)
                put("sdk", Build.VERSION.SDK_INT)
                put("brand", Build.BRAND)
                put("device", Build.DEVICE)
                put("product", Build.PRODUCT)
            }
            Log.d("WebAssemblyApp", "Device info: $info")
            return info.toString()
        }        @JavascriptInterface
        fun showToast(message: String) {
            Log.d("WebAssemblyApp", "Showing toast: $message")
            runOnUiThread {
                Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
            }
        }

        @JavascriptInterface
        fun getBatteryLevel(): Int {
            Log.d("WebAssemblyApp", "Getting battery level")
            return try {
                val batteryManager = getSystemService(Context.BATTERY_SERVICE) as BatteryManager
                val batteryLevel = batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
                Log.d("WebAssemblyApp", "Battery level: $batteryLevel%")
                batteryLevel
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to get battery level: ${e.message}")
                -1
            }
        }

        @JavascriptInterface
        fun getNetworkInfo(): String {
            Log.d("WebAssemblyApp", "Getting network info")
            return try {
                val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
                val networkInfo = JSONObject()
                
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    val activeNetwork = connectivityManager.activeNetwork
                    val networkCapabilities = connectivityManager.getNetworkCapabilities(activeNetwork)
                    
                    if (networkCapabilities != null) {
                        networkInfo.put("connected", true)
                        
                        when {
                            networkCapabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> {
                                networkInfo.put("type", "WiFi")
                                networkInfo.put("isWiFi", true)
                                networkInfo.put("isMobile", false)
                            }
                            networkCapabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> {
                                networkInfo.put("type", "Mobile")
                                networkInfo.put("isWiFi", false)
                                networkInfo.put("isMobile", true)
                            }
                            networkCapabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> {
                                networkInfo.put("type", "Ethernet")
                                networkInfo.put("isWiFi", false)
                                networkInfo.put("isMobile", false)
                            }
                            else -> {
                                networkInfo.put("type", "Unknown")
                                networkInfo.put("isWiFi", false)
                                networkInfo.put("isMobile", false)
                            }
                        }
                        
                        networkInfo.put("isMetered", connectivityManager.isActiveNetworkMetered)
                    } else {
                        networkInfo.put("connected", false)
                        networkInfo.put("type", "None")
                        networkInfo.put("isWiFi", false)
                        networkInfo.put("isMobile", false)
                        networkInfo.put("isMetered", false)
                    }
                } else {
                    @Suppress("DEPRECATION")
                    val activeNetworkInfo = connectivityManager.activeNetworkInfo
                    
                    if (activeNetworkInfo?.isConnected == true) {
                        networkInfo.put("connected", true)
                        networkInfo.put("type", activeNetworkInfo.typeName)
                        networkInfo.put("isWiFi", activeNetworkInfo.type == ConnectivityManager.TYPE_WIFI)
                        networkInfo.put("isMobile", activeNetworkInfo.type == ConnectivityManager.TYPE_MOBILE)
                        networkInfo.put("isMetered", connectivityManager.isActiveNetworkMetered)
                    } else {
                        networkInfo.put("connected", false)
                        networkInfo.put("type", "None")
                        networkInfo.put("isWiFi", false)
                        networkInfo.put("isMobile", false)
                        networkInfo.put("isMetered", false)
                    }
                }
                
                val result = networkInfo.toString()
                Log.d("WebAssemblyApp", "Network info: $result")
                result            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to get network info: ${e.message}")
                "{\"connected\": false, \"type\": \"Error\", \"error\": \"${e.message}\"}"
            }
        }

        @JavascriptInterface
        fun pickImage() {
            Log.d("WebAssemblyApp", "Pick image called")
            openGallery()
        }

        @JavascriptInterface
        fun pickFile() {
            Log.d("WebAssemblyApp", "Pick file called") 
            openFile()
        }

        @JavascriptInterface
        fun startGPS() {
            Log.d("WebAssemblyApp", "Start GPS called")
            getLocation()
        }

        @JavascriptInterface
        fun playSound(soundType: String) {
            Log.d("WebAssemblyApp", "Playing sound: $soundType")
            try {
                // Simple vibration as sound feedback
                vibrate(100)
                Log.d("WebAssemblyApp", "Sound played via vibration")
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to play sound: ${e.message}")
            }
        }

        @JavascriptInterface
        fun showNotification(title: String, message: String) {
            Log.d("WebAssemblyApp", "Showing notification: $title - $message")
            runOnUiThread {
                Toast.makeText(context, "$title: $message", Toast.LENGTH_LONG).show()
            }
        }

        @JavascriptInterface
        fun makeCall(phoneNumber: String) {
            Log.d("WebAssemblyApp", "Making call to: $phoneNumber")
            try {
                val intent = Intent(Intent.ACTION_DIAL).apply {
                    data = Uri.parse("tel:$phoneNumber")
                }
                startActivity(intent)
                Log.d("WebAssemblyApp", "Call intent launched")
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to make call: ${e.message}")
            }
        }

        @JavascriptInterface
        fun sendEmail(email: String, subject: String, body: String) {
            Log.d("WebAssemblyApp", "Sending email to: $email")
            try {
                val intent = Intent(Intent.ACTION_SENDTO).apply {
                    data = Uri.parse("mailto:")
                    putExtra(Intent.EXTRA_EMAIL, arrayOf(email))
                    putExtra(Intent.EXTRA_SUBJECT, subject)
                    putExtra(Intent.EXTRA_TEXT, body)
                }
                startActivity(intent)
                Log.d("WebAssemblyApp", "Email intent launched")
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to send email: ${e.message}")
            }
        }

        @JavascriptInterface
        fun shareContent(content: String, mimeType: String) {
            Log.d("WebAssemblyApp", "Sharing content: $content")
            try {
                val intent = Intent(Intent.ACTION_SEND).apply {
                    type = mimeType
                    putExtra(Intent.EXTRA_TEXT, content)
                }
                startActivity(Intent.createChooser(intent, "Partager via"))
                Log.d("WebAssemblyApp", "Share intent launched")
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to share content: ${e.message}")
            }
        }

        @JavascriptInterface
        fun openBrowser(url: String) {
            Log.d("WebAssemblyApp", "Opening browser with URL: $url")
            try {
                val intent = Intent(Intent.ACTION_VIEW).apply {
                    data = Uri.parse(url)
                }
                startActivity(intent)
                Log.d("WebAssemblyApp", "Browser intent launched")
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to open browser: ${e.message}")
            }
        }

        @JavascriptInterface
        fun closeApp() {
            Log.d("WebAssemblyApp", "Closing application")
            runOnUiThread {
                finish()
            }
        }
    }

    private fun createImageFile(): File {
        val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
        val storageDir = getExternalFilesDir(Environment.DIRECTORY_PICTURES)
        return File.createTempFile(
            "JPEG_${timeStamp}_",
            ".jpg",
            storageDir
        )
    }

    @Deprecated("This method has been deprecated in favor of using the Activity Result API")    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        Log.d("WebAssemblyApp", "onActivityResult: requestCode=$requestCode, resultCode=$resultCode")
        
        when (requestCode) {            
            PICK_IMAGE_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "Image selected: $uri")
                        handleImageSelected(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "Image selection cancelled or failed")
                }
            }
            CAMERA_CAPTURE_REQUEST -> {
                Log.d("WebAssemblyApp", "Camera capture result - resultCode: $resultCode, photoFile exists: ${photoFile?.exists()}")
                
                if (resultCode == Activity.RESULT_OK) {
                    // Utiliser currentPhotoPath si photoFile est null (après restauration d'état)
                    val filePath = photoFile?.absolutePath ?: currentPhotoPath
                    
                    if (filePath != null) {
                        val file = File(filePath)
                        if (file.exists()) {
                            Log.d("WebAssemblyApp", "Photo captured successfully: $filePath")
                            handlePhotoCaptured(filePath)
                        } else {
                            Log.e("WebAssemblyApp", "Photo file does not exist: $filePath")
                            runOnUiThread {
                                Toast.makeText(this, "Erreur: fichier photo introuvable", Toast.LENGTH_SHORT).show()
                            }
                        }
                    } else {
                        Log.e("WebAssemblyApp", "No photo file path available")
                        runOnUiThread {
                            Toast.makeText(this, "Erreur: chemin de fichier photo non disponible", Toast.LENGTH_SHORT).show()
                        }
                    }
                } else {
                    Log.d("WebAssemblyApp", "Photo capture cancelled or failed")
                    runOnUiThread {
                        Toast.makeText(this, "Capture photo annulée", Toast.LENGTH_SHORT).show()
                    }
                }
                
                // Nettoyer les variables
                currentPhotoPath = null
            }
            3 -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "Video recorded: $uri")
                        handleVideoRecorded(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "Video recording cancelled or failed")
                }
            }
            1001 -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "File selected: $uri")
                        handleFileSelected(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "File selection cancelled or failed")
                }
            }
        }
    }// Méthodes de traitement des résultats d'activité en Kotlin pur
    private fun handleImageSelected(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing selected image: $uri")
        try {
            // Traitement de l'image sélectionnée
            val imageInfo = JSONObject().apply {
                put("type", "image")
                put("uri", uri.toString())
                put("timestamp", System.currentTimeMillis())
            }
            
            Log.d("WebAssemblyApp", "Image info: $imageInfo")
            
            // Afficher un toast avec les informations
            runOnUiThread {
                Toast.makeText(this, "Image sélectionnée: ${uri.lastPathSegment}", Toast.LENGTH_SHORT).show()
            }
            
            // Traitement Kotlin uniquement - pas de JavaScript
            processImageFile(uri)
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing selected image: ${e.message}")
        }
    }
    
    private fun handlePhotoCaptured(filePath: String) {
        Log.d("WebAssemblyApp", "Processing captured photo: $filePath")
        try {
            val file = File(filePath)
            if (file.exists()) {
                val photoInfo = JSONObject().apply {
                    put("type", "photo")
                    put("path", filePath)
                    put("size", file.length())
                    put("timestamp", System.currentTimeMillis())
                }
                
                Log.d("WebAssemblyApp", "Photo info: $photoInfo")
                
                // Afficher un toast avec les informations
                runOnUiThread {
                    Toast.makeText(this, "Photo capturée: ${file.name} (${file.length()} bytes)", Toast.LENGTH_LONG).show()
                }
                
                // Traitement Kotlin uniquement - pas de JavaScript
                processPhotoFile(file)
                
            } else {
                Log.w("WebAssemblyApp", "Captured photo file does not exist: $filePath")
                runOnUiThread {
                    Toast.makeText(this, "Erreur: fichier photo introuvable", Toast.LENGTH_SHORT).show()
                }
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing captured photo: ${e.message}")
        }
    }
    
    private fun handleVideoRecorded(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing recorded video: $uri")
        try {
            val videoInfo = JSONObject().apply {
                put("type", "video")
                put("uri", uri.toString())
                put("timestamp", System.currentTimeMillis())
            }
            
            Log.d("WebAssemblyApp", "Video info: $videoInfo")
            
            // Afficher un toast avec les informations
            runOnUiThread {
                Toast.makeText(this, "Vidéo enregistrée: ${uri.lastPathSegment}", Toast.LENGTH_LONG).show()
            }
            
            // Traitement Kotlin uniquement - pas de JavaScript
            processVideoFile(uri)
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing recorded video: ${e.message}")
        }
    }
    
    private fun handleFileSelected(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing selected file: $uri")
        try {
            val fileInfo = JSONObject().apply {
                put("type", "file")
                put("uri", uri.toString())
                put("timestamp", System.currentTimeMillis())
            }
            
            Log.d("WebAssemblyApp", "File info: $fileInfo")
            
            // Afficher un toast avec les informations
            runOnUiThread {
                Toast.makeText(this, "Fichier sélectionné: ${uri.lastPathSegment}", Toast.LENGTH_SHORT).show()
            }
            
            // Traitement Kotlin uniquement - pas de JavaScript
            processSelectedFile(uri)
              } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing selected file: ${e.message}")
        }
    }

    // Méthodes de traitement spécifiques en Kotlin pur
    private fun processImageFile(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing image file in Kotlin: $uri")
        try {
            // Copier l'image dans la galerie du téléphone
            val inputStream = contentResolver.openInputStream(uri)
            val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
            val fileName = "IMG_${timeStamp}.jpg"
            
            // Utiliser le dossier Pictures public pour que l'image apparaisse dans la galerie
            val picturesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES)
            val appDir = File(picturesDir, "WebAssemblyApp")
            appDir.mkdirs()
            
            val outputFile = File(appDir, fileName)
            
            inputStream?.use { input ->
                FileOutputStream(outputFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            Log.d("WebAssemblyApp", "Image saved to gallery: ${outputFile.absolutePath}")
            
            // Notifier le système que le fichier a été ajouté pour qu'il apparaisse dans la galerie
            val intent = Intent(Intent.ACTION_MEDIA_SCANNER_SCAN_FILE)
            intent.data = Uri.fromFile(outputFile)
            sendBroadcast(intent)
            
            runOnUiThread {
                Toast.makeText(this, "🖼️ Image sauvée dans la galerie: ${outputFile.name}", Toast.LENGTH_LONG).show()
            }
            
            // Optionnel: Redimensionner l'image si trop grande
            resizeImageIfNeeded(outputFile)
              } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing image: ${e.message}")
            runOnUiThread {
                Toast.makeText(this, "Erreur de traitement image: ${e.message}", Toast.LENGTH_SHORT).show()
            }
        }
    }

    private fun processPhotoFile(file: File) {
        Log.d("WebAssemblyApp", "Processing photo file in Kotlin: ${file.absolutePath}")
        try {
            if (file.exists()) {
                // Sauvegarder dans la galerie du téléphone
                val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
                val fileName = "PHOTO_${timeStamp}.jpg"
                
                // Utiliser le dossier Pictures public pour que l'image apparaisse dans la galerie
                val picturesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES)
                val appDir = File(picturesDir, "WebAssemblyApp")
                appDir.mkdirs()
                
                val permanentFile = File(appDir, fileName)
                
                file.copyTo(permanentFile, overwrite = true)
                
                Log.d("WebAssemblyApp", "Photo copied to gallery: ${permanentFile.absolutePath}")
                
                // Notifier le système que le fichier a été ajouté pour qu'il apparaisse dans la galerie
                val intent = Intent(Intent.ACTION_MEDIA_SCANNER_SCAN_FILE)
                intent.data = Uri.fromFile(permanentFile)
                sendBroadcast(intent)
                
                runOnUiThread {
                    Toast.makeText(this, "📷 Photo sauvée dans la galerie: ${permanentFile.name}", Toast.LENGTH_LONG).show()
                }
                
                // Optionnel: Redimensionner la photo si nécessaire
                resizeImageIfNeeded(permanentFile)
                
                // Supprimer le fichier temporaire si ce n'est pas le fichier permanent
                if (file.absolutePath != permanentFile.absolutePath) {
                    file.delete()
                    Log.d("WebAssemblyApp", "Temporary file deleted: ${file.absolutePath}")
                }
                
            } else {
                Log.w("WebAssemblyApp", "Photo file does not exist: ${file.absolutePath}")
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing photo: ${e.message}")
            runOnUiThread {
                Toast.makeText(this, "Erreur de traitement photo: ${e.message}", Toast.LENGTH_SHORT).show()
            }
        }
    }
    
    private fun processVideoFile(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing video file in Kotlin: $uri")
        try {
            // Copier la vidéo dans le répertoire de l'app
            val inputStream = contentResolver.openInputStream(uri)
            val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
            val fileName = "VID_${timeStamp}.mp4"
            val outputFile = File(getExternalFilesDir(Environment.DIRECTORY_MOVIES), fileName)
            
            // Créer le répertoire s'il n'existe pas
            outputFile.parentFile?.mkdirs()
            
            inputStream?.use { input ->
                FileOutputStream(outputFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            Log.d("WebAssemblyApp", "Video saved to: ${outputFile.absolutePath}")
            
            runOnUiThread {
                Toast.makeText(this, "Vidéo sauvegardée: ${outputFile.name} (${formatFileSize(outputFile.length())})", Toast.LENGTH_LONG).show()
            }
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing video: ${e.message}")
            runOnUiThread {
                Toast.makeText(this, "Erreur de traitement vidéo: ${e.message}", Toast.LENGTH_SHORT).show()
            }
        }
    }
    
    private fun processSelectedFile(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing selected file in Kotlin: $uri")
        try {
            // Obtenir le nom du fichier
            val fileName = getFileNameFromUri(uri)
            val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
            val safeFileName = "${timeStamp}_${fileName}"
            
            // Copier le fichier dans le répertoire de l'app
            val inputStream = contentResolver.openInputStream(uri)
            val outputFile = File(getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS), safeFileName)
            
            // Créer le répertoire s'il n'existe pas
            outputFile.parentFile?.mkdirs()
            
            inputStream?.use { input ->
                FileOutputStream(outputFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            Log.d("WebAssemblyApp", "File saved to: ${outputFile.absolutePath}")
            
            runOnUiThread {
                Toast.makeText(this, "Fichier sauvegardé: ${outputFile.name} (${formatFileSize(outputFile.length())})", Toast.LENGTH_LONG).show()
            }
            
            // Analyser le type de fichier et effectuer un traitement spécifique
            analyzeAndProcessFile(outputFile)
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing file: ${e.message}")
            runOnUiThread {
                Toast.makeText(this, "Erreur de traitement fichier: ${e.message}", Toast.LENGTH_SHORT).show()
            }
        }
    }
    
    // Méthodes utilitaires
    private fun formatFileSize(bytes: Long): String {
        return when {
            bytes < 1024 -> "$bytes B"
            bytes < 1024 * 1024 -> "${bytes / 1024} KB"
            bytes < 1024 * 1024 * 1024 -> "${bytes / (1024 * 1024)} MB"
            else -> "${bytes / (1024 * 1024 * 1024)} GB"
        }
    }
    
    private fun resizeImageIfNeeded(imageFile: File) {
        try {
            // Vérifier la taille du fichier (si > 2MB, redimensionner)
            if (imageFile.length() > 2 * 1024 * 1024) {
                Log.d("WebAssemblyApp", "Image is large (${formatFileSize(imageFile.length())}), considering resize")
                // Ici vous pourriez implémenter une logique de redimensionnement
                // avec BitmapFactory et Bitmap.createScaledBitmap()
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error checking image size: ${e.message}")
        }
    }
    
    private fun getFileNameFromUri(uri: Uri): String {
        var fileName = "unknown_file"
        try {
            contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val nameIndex = cursor.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                    if (nameIndex >= 0) {
                        fileName = cursor.getString(nameIndex) ?: "unknown_file"
                    }
                }
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error getting file name: ${e.message}")
            fileName = "file_${System.currentTimeMillis()}"
        }
        return fileName
    }
    
    private fun analyzeAndProcessFile(file: File) {
        try {
            val extension = file.extension.lowercase()
            Log.d("WebAssemblyApp", "Analyzing file type: $extension")
            
            when (extension) {
                "txt", "log" -> {
                    Log.d("WebAssemblyApp", "Text file detected, reading content preview")
                    // Lire les premières lignes du fichier texte
                    val preview = file.readLines().take(3).joinToString("\n")
                    Log.d("WebAssemblyApp", "File preview: $preview")
                }
                "json" -> {
                    Log.d("WebAssemblyApp", "JSON file detected, validating structure")
                    // Valider la structure JSON
                    try {
                        val content = file.readText()
                        JSONObject(content)
                        Log.d("WebAssemblyApp", "Valid JSON file")
                    } catch (e: Exception) {
                        Log.w("WebAssemblyApp", "Invalid JSON file: ${e.message}")
                    }
                }
                "pdf" -> {
                    Log.d("WebAssemblyApp", "PDF file detected")
                    // Ici vous pourriez ajouter un traitement PDF spécifique
                }
                "zip", "rar", "7z" -> {
                    Log.d("WebAssemblyApp", "Archive file detected")
                    // Ici vous pourriez ajouter un traitement d'archive
                }
                else -> {
                    Log.d("WebAssemblyApp", "Unknown file type: $extension")
                }            }        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error analyzing file: ${e.message}")
        }
    }

    private fun copyAudioToPublicDirectory(sourceFile: File): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                // Android 10+ : Utiliser MediaStore API
                val resolver = contentResolver
                val audioCollection = MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
                
                val audioDetails = ContentValues().apply {
                    put(MediaStore.Audio.Media.DISPLAY_NAME, sourceFile.name)
                    put(MediaStore.Audio.Media.MIME_TYPE, "audio/3gpp")
                    put(MediaStore.Audio.Media.RELATIVE_PATH, "Music/WebAssemblyApp/")
                    put(MediaStore.Audio.Media.IS_PENDING, 1)
                }
                
                val audioUri = resolver.insert(audioCollection, audioDetails)
                
                if (audioUri != null) {
                    resolver.openOutputStream(audioUri)?.use { outputStream ->
                        sourceFile.inputStream().use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    
                    // Marquer comme non-pending
                    audioDetails.clear()
                    audioDetails.put(MediaStore.Audio.Media.IS_PENDING, 0)
                    resolver.update(audioUri, audioDetails, null, null)
                    
                    Log.d("WebAssemblyApp", "Audio saved via MediaStore: $audioUri")
                    audioUri.toString()
                } else {
                    Log.e("WebAssemblyApp", "Failed to create MediaStore entry")
                    null
                }
            } else {
                // Android 9 et antérieur : Copie directe vers le répertoire public
                val musicDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MUSIC)
                val appDir = File(musicDir, "WebAssemblyApp")
                appDir.mkdirs()
                
                val publicFile = File(appDir, sourceFile.name)
                sourceFile.copyTo(publicFile, overwrite = true)
                
                // Notifier le système que le fichier a été ajouté
                val intent = Intent(Intent.ACTION_MEDIA_SCANNER_SCAN_FILE)
                intent.data = Uri.fromFile(publicFile)
                sendBroadcast(intent)
                
                Log.d("WebAssemblyApp", "Audio copied to public directory: ${publicFile.absolutePath}")
                publicFile.absolutePath
            }        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error copying audio to public directory: ${e.message}")
            null
        }
    }

    private fun copyVideoToPublicDirectory(sourceFile: File): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                // Android 10+ : Utiliser MediaStore API
                val resolver = contentResolver
                val videoCollection = MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                
                val videoDetails = ContentValues().apply {
                    put(MediaStore.Video.Media.DISPLAY_NAME, sourceFile.name)
                    put(MediaStore.Video.Media.MIME_TYPE, "video/mp4")
                    put(MediaStore.Video.Media.RELATIVE_PATH, "Movies/WebAssemblyApp/")
                    put(MediaStore.Video.Media.IS_PENDING, 1)
                }
                
                val videoUri = resolver.insert(videoCollection, videoDetails)
                
                if (videoUri != null) {
                    resolver.openOutputStream(videoUri)?.use { outputStream ->
                        sourceFile.inputStream().use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    
                    // Marquer comme non-pending
                    videoDetails.clear()
                    videoDetails.put(MediaStore.Video.Media.IS_PENDING, 0)
                    resolver.update(videoUri, videoDetails, null, null)
                    
                    Log.d("WebAssemblyApp", "Video saved via MediaStore: $videoUri")
                    videoUri.toString()
                } else {
                    Log.e("WebAssemblyApp", "Failed to create MediaStore entry for video")
                    null
                }
            } else {
                // Android 9 et antérieur : Copie directe vers le répertoire public
                val moviesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MOVIES)
                val appDir = File(moviesDir, "WebAssemblyApp")
                appDir.mkdirs()
                
                val publicFile = File(appDir, sourceFile.name)
                sourceFile.copyTo(publicFile, overwrite = true)
                
                // Notifier le système que le fichier a été ajouté
                val intent = Intent(Intent.ACTION_MEDIA_SCANNER_SCAN_FILE)
                intent.data = Uri.fromFile(publicFile)
                sendBroadcast(intent)
                
                Log.d("WebAssemblyApp", "Video copied to public directory: ${publicFile.absolutePath}")
                publicFile.absolutePath
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error copying video to public directory: ${e.message}")
            null
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d("WebAssemblyApp", "MainActivity onDestroy")          // Libérer les ressources
        try {
            mediaRecorder?.release()
            videoMediaRecorder?.release()
            camera?.apply {
                try {
                    stopPreview()
                    lock()
                    release()
                } catch (e: Exception) {
                    Log.w("WebAssemblyApp", "Error stopping camera in onDestroy: ${e.message}")
                    release()
                }
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error releasing resources: ${e.message}")
        }
        
        mediaRecorder = null
        videoMediaRecorder = null
        camera = null
    }

    override fun onPause() {
        super.onPause()
        Log.d("WebAssemblyApp", "MainActivity onPause")
        
        // Sauvegarder l'URL actuelle du WebView
        if (::webView.isInitialized) {
            webViewUrl = webView.url
        }
    }

    override fun onResume() {
        super.onResume()
        Log.d("WebAssemblyApp", "MainActivity onResume")
        
        // Reprendre le WebView si nécessaire
        if (::webView.isInitialized) {
            webView.onResume()
        }
    }

    override fun onLowMemory() {
        super.onLowMemory()
        Log.w("WebAssemblyApp", "Low memory warning")
        
        // Libérer les ressources non essentielles
        if (::webView.isInitialized) {
            webView.freeMemory()
        }
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        
        // Sauvegarder l'état important
        currentPhotoPath?.let { path ->
            outState.putString("currentPhotoPath", path)
        }
        
        if (::webView.isInitialized) {
            webViewUrl = webView.url
            webViewUrl?.let { url ->
                outState.putString("webViewUrl", url)
            }
        }
        
        pendingAction?.let { action ->
            outState.putString("pendingAction", action)
        }
        
        pendingSmsNumber?.let { number ->
            outState.putString("pendingSmsNumber", number)
        }
        
        pendingSmsMessage?.let { message ->
            outState.putString("pendingSmsMessage", message)
        }
        
        Log.d("WebAssemblyApp", "State saved - currentPhotoPath: $currentPhotoPath")
    }

    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
        
        // Restaurer l'URL du WebView si elle a été sauvegardée
        webViewUrl?.let { url ->
            if (::webView.isInitialized && url != webView.url) {
                Log.d("WebAssemblyApp", "Restoring WebView URL: $url")
                webView.loadUrl(url)
            }
        }
        
        Log.d("WebAssemblyApp", "Instance state restored")
    }
}
