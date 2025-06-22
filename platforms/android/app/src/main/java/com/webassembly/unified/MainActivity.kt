package com.webassembly.unified

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
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

class MainActivity : AppCompatActivity() {
    
    private lateinit var webView: WebView
    private var mediaRecorder: MediaRecorder? = null
    private var camera: Camera? = null
    private var previewLayout: FrameLayout? = null
    private var isRecording = false
    private var outputFile: File? = null
    private var photoFile: File? = null
    
    // Variables pour mémoriser les actions en attente de permission
    private var pendingAction: String? = null
    private var pendingSmsNumber: String? = null
    private var pendingSmsMessage: String? = null
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
                    }
                }
                "startRecording" -> {
                    pendingAction = null
                    findViewById<WebView>(R.id.webview).post {
                        (findViewById<WebView>(R.id.webview).getTag() as? WebAppInterface)?.executeAudioRecording()
                            ?: WebAppInterface(this).executeAudioRecording()
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
        
        // Keep screen on
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        
        setContentView(R.layout.activity_main)
        
        setupWebView()
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
        }

        @JavascriptInterface
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
                        requestPermissionLauncher.launch(Manifest.permission.READ_EXTERNAL_STORAGE)
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
                    } else {
                        ContextCompat.checkSelfPermission(context, Manifest.permission.READ_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
                    }                }
                else -> false
            }
            Log.d("WebAssemblyApp", "Permission $permission: $result")
            return result
        }

        @JavascriptInterface
        fun startRecording() {
            Log.d("WebAssemblyApp", "Starting audio recording")            
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) 
                == PackageManager.PERMISSION_GRANTED) {
                executeAudioRecording()
            } else {
                Log.d("WebAssemblyApp", "Requesting microphone permission for recording")
                pendingAction = "startRecording"
                requestPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
            }
        }

        fun executeAudioRecording() {
            try {
                outputFile = File(context.externalCacheDir, "recording_${System.currentTimeMillis()}.3gp")
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
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Recording failed: ${e.message}")
            }
        }

        @JavascriptInterface
        fun stopRecording(): String {
            Log.d("WebAssemblyApp", "Stopping audio recording")
            return try {
                mediaRecorder?.apply {
                    stop()
                    release()
                }
                mediaRecorder = null
                this@MainActivity.isRecording = false
                val filePath = outputFile?.absolutePath ?: ""
                Log.d("WebAssemblyApp", "Recording stopped: $filePath")
                filePath
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Stop recording failed: ${e.message}")
                ""
            }
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
                
                return executeGetLocation()
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
        }

        // Méthodes d'exécution séparées
        fun executeCamera() {
            try {
                photoFile = createImageFile()
                val photoURI = FileProvider.getUriForFile(
                    this@MainActivity,
                    "com.webassembly.unified.fileprovider",
                    photoFile!!
                )
                
                val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
                    putExtra(MediaStore.EXTRA_OUTPUT, photoURI)
                }
                startActivityForResult(intent, CAMERA_CAPTURE_REQUEST)
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
            }
        }

        fun executeVideoRecording() {
            try {
                val intent = Intent(MediaStore.ACTION_VIDEO_CAPTURE)
                startActivityForResult(intent, 3)
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
            }
        }

        @JavascriptInterface
        fun openCamera() {
            Log.d("WebAssemblyApp", "Opening camera")
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) 
                == PackageManager.PERMISSION_GRANTED) {
                
                try {
                    photoFile = createImageFile()
                    val photoURI = FileProvider.getUriForFile(
                        this@MainActivity,
                        "com.webassembly.unified.fileprovider",
                        photoFile!!
                    )
                    
                    val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
                        putExtra(MediaStore.EXTRA_OUTPUT, photoURI)
                    }
                    startActivityForResult(intent, CAMERA_CAPTURE_REQUEST)
                } catch (e: Exception) {
                    Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
                }            } else {
                Log.e("WebAssemblyApp", "No permission for camera")
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
                result
            } catch (e: Exception) {
                Log.e("WebAssemblyApp", "Failed to get network info: ${e.message}")                
                "${e.message}"
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
        fun recordAudio() {
            Log.d("WebAssemblyApp", "Record audio called")
            startRecording()
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

    @Deprecated("This method has been deprecated in favor of using the Activity Result API")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        Log.d("WebAssemblyApp", "onActivityResult: requestCode=$requestCode, resultCode=$resultCode")
        
        when (requestCode) {            PICK_IMAGE_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "Image selected: $uri")
                        webView.evaluateJavascript("window.handleImageSelected('${uri}');", null)
                    }
                }
            }
            CAMERA_CAPTURE_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    photoFile?.let { file ->
                        Log.d("WebAssemblyApp", "Photo captured: ${file.absolutePath}")
                        webView.evaluateJavascript("window.handlePhotoCaptured('${file.absolutePath}');", null)
                    }
                }
            }
            3 -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "Video recorded: $uri")
                        webView.evaluateJavascript("window.handleVideoRecorded('${uri}');", null)
                    }
                }
            }
            1001 -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        Log.d("WebAssemblyApp", "File selected: $uri")
                        webView.evaluateJavascript("window.handleFileSelected('${uri}');", null)
                    }
                }
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d("WebAssemblyApp", "MainActivity onDestroy")
        mediaRecorder?.release()
        camera?.release()
    }

    override fun onPause() {
        super.onPause()
        Log.d("WebAssemblyApp", "MainActivity onPause")
    }

    override fun onResume() {
        super.onResume()
        Log.d("WebAssemblyApp", "MainActivity onResume")
    }
}
