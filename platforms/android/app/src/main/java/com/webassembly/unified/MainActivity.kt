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
import android.net.Uri
import android.os.Vibrator
import android.os.VibrationEffect
import android.os.Build
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import androidx.core.app.NotificationCompat
import android.location.LocationManager
import android.location.Location
import android.location.LocationListener
import android.os.BatteryManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.telephony.SmsManager
import android.media.MediaRecorder
import android.media.MediaPlayer
import android.os.Environment
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import android.content.ActivityNotFoundException

class MainActivity : Activity() {
    
    private lateinit var webView: WebView
    
    // Request codes
    private val CAMERA_REQUEST = 1888
    private val VIDEO_REQUEST = 1889
    private val PICK_IMAGE_REQUEST = 1890
    private val PICK_FILE_REQUEST = 1891
    private val AUDIO_REQUEST = 1892
    
    // Permission codes
    private val CAMERA_PERMISSION_CODE = 100
    private val LOCATION_PERMISSION_CODE = 101
    private val WRITE_STORAGE_PERMISSION_CODE = 102
    private val RECORD_AUDIO_PERMISSION_CODE = 103
    private val SMS_PERMISSION_CODE = 104
    private val CALL_PERMISSION_CODE = 105
    
    // Notification
    private val NOTIFICATION_CHANNEL_ID = "WebAssembly_Channel"
    private val NOTIFICATION_ID = 1001
    
    // Media recorder
    private var mediaRecorder: MediaRecorder? = null
    private var audioFilePath: String? = null
    private var isRecording = false
    
    // Location
    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null
    
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
        
        // Initialize notification channel
        createNotificationChannel()
        
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
    
    // ========== NOTIFICATION SETUP ==========
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "WebAssembly Notifications",
                NotificationManager.IMPORTANCE_DEFAULT
            )
            channel.description = "Notifications from WebAssembly App"
            
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            notificationManager.createNotificationChannel(channel)
        }
    }
    
    // ========== PERMISSION CHECKS ==========
    private fun checkCameraPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun checkLocationPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun checkStoragePermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.WRITE_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun checkAudioPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun checkSmsPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.SEND_SMS) == PackageManager.PERMISSION_GRANTED
    }
    
    private fun checkCallPermission(): Boolean {
        return ContextCompat.checkSelfPermission(this, Manifest.permission.CALL_PHONE) == PackageManager.PERMISSION_GRANTED
    }
    
    // ========== PERMISSION REQUESTS ==========
    private fun requestCameraPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CAMERA), CAMERA_PERMISSION_CODE)
    }
    
    private fun requestLocationPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        ), LOCATION_PERMISSION_CODE)
    }
    
    private fun requestStoragePermission() {
        ActivityCompat.requestPermissions(this, arrayOf(
            Manifest.permission.WRITE_EXTERNAL_STORAGE,
            Manifest.permission.READ_EXTERNAL_STORAGE
        ), WRITE_STORAGE_PERMISSION_CODE)
    }
    
    private fun requestAudioPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.RECORD_AUDIO), RECORD_AUDIO_PERMISSION_CODE)
    }
    
    private fun requestSmsPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.SEND_SMS), SMS_PERMISSION_CODE)
    }
    
    private fun requestCallPermission() {
        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CALL_PHONE), CALL_PERMISSION_CODE)
    }
    
    // ========== CAMERA AND MEDIA ==========
    private fun openCamera() {
        val cameraIntent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
        if (cameraIntent.resolveActivity(packageManager) != null) {
            startActivityForResult(cameraIntent, CAMERA_REQUEST)
        } else {
            Toast.makeText(this, "Aucune application caméra trouvée", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun recordVideo() {
        val videoIntent = Intent(MediaStore.ACTION_VIDEO_CAPTURE)
        if (videoIntent.resolveActivity(packageManager) != null) {
            startActivityForResult(videoIntent, VIDEO_REQUEST)
        } else {
            Toast.makeText(this, "Aucune application vidéo trouvée", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun pickImage() {
        val intent = Intent(Intent.ACTION_PICK, MediaStore.Images.Media.EXTERNAL_CONTENT_URI)
        startActivityForResult(intent, PICK_IMAGE_REQUEST)
    }
    
    private fun pickFile() {
        val intent = Intent(Intent.ACTION_GET_CONTENT)
        intent.type = "*/*"
        intent.addCategory(Intent.CATEGORY_OPENABLE)
        startActivityForResult(Intent.createChooser(intent, "Choisir un fichier"), PICK_FILE_REQUEST)
    }
    
    // ========== LOCATION ==========
    private fun getCurrentLocation() {
        if (!checkLocationPermission()) {
            requestLocationPermission()
            return
        }
        
        locationManager = getSystemService(Context.LOCATION_SERVICE) as LocationManager
        locationListener = object : LocationListener {
            override fun onLocationChanged(location: Location) {
                val lat = location.latitude
                val lng = location.longitude
                val accuracy = location.accuracy
                
                webView.evaluateJavascript(
                    "if(window.onLocationReceived) window.onLocationReceived($lat, $lng, $accuracy);",
                    null
                )
                Toast.makeText(this@MainActivity, "Position: $lat, $lng", Toast.LENGTH_SHORT).show()
                locationManager?.removeUpdates(this)
            }
            
            override fun onStatusChanged(provider: String?, status: Int, extras: Bundle?) {}
            override fun onProviderEnabled(provider: String) {}
            override fun onProviderDisabled(provider: String) {}
        }
        
        try {
            locationManager?.requestLocationUpdates(
                LocationManager.GPS_PROVIDER,
                1000L,
                1f,
                locationListener!!
            )
        } catch (e: SecurityException) {
            Toast.makeText(this, "Erreur d'accès à la localisation", Toast.LENGTH_SHORT).show()
        }
    }
    
    // ========== DEVICE INFO ==========
    private fun getBatteryLevel(): Int {
        val batteryManager = getSystemService(Context.BATTERY_SERVICE) as BatteryManager
        return batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
    }
    
    private fun getNetworkInfo(): String {
        val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        val network = connectivityManager.activeNetwork
        val capabilities = connectivityManager.getNetworkCapabilities(network)
        
        return when {
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true -> "WiFi"
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true -> "Mobile"
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) == true -> "Ethernet"
            else -> "Déconnecté"
        }
    }
    
    private fun getDeviceInfo(): String {
        return "Modèle: ${Build.MODEL}, Android: ${Build.VERSION.RELEASE}, SDK: ${Build.VERSION.SDK_INT}"
    }
    
    // ========== AUDIO ==========
    private fun startAudioRecording() {
        if (!checkAudioPermission()) {
            requestAudioPermission()
            return
        }
        
        if (isRecording) {
            stopAudioRecording()
            return
        }
        
        try {
            audioFilePath = "${externalCacheDir?.absolutePath}/audio_record.3gp"
            mediaRecorder = MediaRecorder().apply {
                setAudioSource(MediaRecorder.AudioSource.MIC)
                setOutputFormat(MediaRecorder.OutputFormat.THREE_GPP)
                setOutputFile(audioFilePath)
                setAudioEncoder(MediaRecorder.AudioEncoder.AMR_NB)
                prepare()
                start()
            }
            isRecording = true
            Toast.makeText(this, "Enregistrement audio démarré", Toast.LENGTH_SHORT).show()
        } catch (e: IOException) {
            Log.e(TAG, "Erreur enregistrement audio: ${e.message}")
            Toast.makeText(this, "Erreur enregistrement audio", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun stopAudioRecording() {
        if (isRecording) {
            mediaRecorder?.apply {
                stop()
                release()
            }
            mediaRecorder = null
            isRecording = false
            Toast.makeText(this, "Enregistrement audio arrêté", Toast.LENGTH_SHORT).show()
            
            webView.evaluateJavascript(
                "if(window.onAudioRecorded) window.onAudioRecorded('$audioFilePath');",
                null
            )
        }
    }
    
    private fun playSound() {
        try {
            val mediaPlayer = MediaPlayer.create(this, android.provider.Settings.System.DEFAULT_NOTIFICATION_URI)
            mediaPlayer?.start()
            mediaPlayer?.setOnCompletionListener { it.release() }
        } catch (e: Exception) {
            Log.e(TAG, "Erreur lecture son: ${e.message}")
        }
    }
    
    // ========== VIBRATION AND NOTIFICATIONS ==========
    private fun vibrateDevice(duration: Long) {
        val vibrator = getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(duration)
        }
    }
    
    private fun showNotification(title: String, message: String) {
        val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        
        val notification = NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle(title)
            .setContentText(message)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()
        
        notificationManager.notify(NOTIFICATION_ID, notification)
    }
    
    private fun showToast(message: String) {
        runOnUiThread {
            Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
        }
    }
    
    // ========== COMMUNICATION ==========
    private fun sendSms(phoneNumber: String, message: String) {
        if (!checkSmsPermission()) {
            requestSmsPermission()
            return
        }
        
        try {
            val smsManager = SmsManager.getDefault()
            smsManager.sendTextMessage(phoneNumber, null, message, null, null)
            Toast.makeText(this, "SMS envoyé", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e(TAG, "Erreur envoi SMS: ${e.message}")
            Toast.makeText(this, "Erreur envoi SMS", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun makePhoneCall(phoneNumber: String) {
        if (!checkCallPermission()) {
            requestCallPermission()
            return
        }
        
        try {
            val intent = Intent(Intent.ACTION_CALL)
            intent.data = Uri.parse("tel:$phoneNumber")
            startActivity(intent)
        } catch (e: Exception) {
            Log.e(TAG, "Erreur appel: ${e.message}")
            Toast.makeText(this, "Erreur lors de l'appel", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun sendEmail(recipient: String, subject: String, body: String) {
        try {
            val intent = Intent(Intent.ACTION_SENDTO)
            intent.data = Uri.parse("mailto:")
            intent.putExtra(Intent.EXTRA_EMAIL, arrayOf(recipient))
            intent.putExtra(Intent.EXTRA_SUBJECT, subject)
            intent.putExtra(Intent.EXTRA_TEXT, body)
            
            if (intent.resolveActivity(packageManager) != null) {
                startActivity(intent)
            } else {
                Toast.makeText(this, "Aucune application email trouvée", Toast.LENGTH_SHORT).show()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Erreur envoi email: ${e.message}")
            Toast.makeText(this, "Erreur envoi email", Toast.LENGTH_SHORT).show()
        }
    }
    
    // ========== SYSTEM ==========
    private fun shareContent(content: String) {
        val intent = Intent(Intent.ACTION_SEND)
        intent.type = "text/plain"
        intent.putExtra(Intent.EXTRA_TEXT, content)
        startActivity(Intent.createChooser(intent, "Partager via"))
    }
    
    private fun openBrowser(url: String) {
        try {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
            startActivity(intent)
        } catch (e: ActivityNotFoundException) {
            Toast.makeText(this, "Aucun navigateur trouvé", Toast.LENGTH_SHORT).show()
        }
    }
    
    private fun saveFile(fileName: String, content: String) {
        if (!checkStoragePermission()) {
            requestStoragePermission()
            return
        }
        
        try {
            val file = File(externalCacheDir, fileName)
            FileOutputStream(file).use { fos ->
                fos.write(content.toByteArray())
            }
            Toast.makeText(this, "Fichier sauvegardé: ${file.absolutePath}", Toast.LENGTH_SHORT).show()
            
            webView.evaluateJavascript(
                "if(window.onFileSaved) window.onFileSaved('${file.absolutePath}');",
                null
            )
        } catch (e: IOException) {
            Log.e(TAG, "Erreur sauvegarde: ${e.message}")
            Toast.makeText(this, "Erreur sauvegarde fichier", Toast.LENGTH_SHORT).show()        }
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
            LOCATION_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    getCurrentLocation()
                } else {
                    Toast.makeText(this, "Permission localisation refusée", Toast.LENGTH_SHORT).show()
                }
            }
            WRITE_STORAGE_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    Toast.makeText(this, "Permission stockage accordée", Toast.LENGTH_SHORT).show()
                } else {
                    Toast.makeText(this, "Permission stockage refusée", Toast.LENGTH_SHORT).show()
                }
            }
            RECORD_AUDIO_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    startAudioRecording()
                } else {
                    Toast.makeText(this, "Permission audio refusée", Toast.LENGTH_SHORT).show()
                }
            }
            SMS_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    Toast.makeText(this, "Permission SMS accordée", Toast.LENGTH_SHORT).show()
                } else {
                    Toast.makeText(this, "Permission SMS refusée", Toast.LENGTH_SHORT).show()
                }
            }
            CALL_PERMISSION_CODE -> {
                if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                    Toast.makeText(this, "Permission appel accordée", Toast.LENGTH_SHORT).show()
                } else {
                    Toast.makeText(this, "Permission appel refusée", Toast.LENGTH_SHORT).show()
                }
            }
        }
    }
    
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)

        when (requestCode) {
            CAMERA_REQUEST -> {
                if (resultCode == RESULT_OK) {
                    val photo = data?.extras?.get("data") as? Bitmap
                    if (photo != null) {
                        Log.i(TAG, "Photo prise avec succès")
                        webView.evaluateJavascript(
                            "if(window.onPhotoTaken) window.onPhotoTaken('Photo prise avec succès');",
                            null
                        )
                        Toast.makeText(this, "Photo prise !", Toast.LENGTH_SHORT).show()
                    }
                }
            }
            VIDEO_REQUEST -> {
                if (resultCode == RESULT_OK) {
                    Log.i(TAG, "Vidéo enregistrée avec succès")
                    webView.evaluateJavascript(
                        "if(window.onVideoRecorded) window.onVideoRecorded('Vidéo enregistrée avec succès');",
                        null
                    )
                    Toast.makeText(this, "Vidéo enregistrée !", Toast.LENGTH_SHORT).show()
                }
            }
            PICK_IMAGE_REQUEST -> {
                if (resultCode == RESULT_OK && data != null) {
                    val selectedImageUri = data.data
                    Log.i(TAG, "Image sélectionnée: $selectedImageUri")
                    webView.evaluateJavascript(
                        "if(window.onImagePicked) window.onImagePicked('$selectedImageUri');",
                        null
                    )
                    Toast.makeText(this, "Image sélectionnée !", Toast.LENGTH_SHORT).show()
                }
            }
            PICK_FILE_REQUEST -> {
                if (resultCode == RESULT_OK && data != null) {
                    val selectedFileUri = data.data
                    Log.i(TAG, "Fichier sélectionné: $selectedFileUri")
                    webView.evaluateJavascript(
                        "if(window.onFilePicked) window.onFilePicked('$selectedFileUri');",
                        null
                    )
                    Toast.makeText(this, "Fichier sélectionné !", Toast.LENGTH_SHORT).show()
                }
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
        
        // ========== CAMERA AND MEDIA ==========
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
        
        @JavascriptInterface
        fun recordVideo() {
            Log.d(TAG, "Record video button clicked from JavaScript")
            runOnUiThread {
                if (checkCameraPermission()) {
                    recordVideo()
                } else {
                    requestCameraPermission()
                }
            }
        }
        
        @JavascriptInterface
        fun pickImage() {
            Log.d(TAG, "Pick image button clicked from JavaScript")
            runOnUiThread {
                pickImage()
            }
        }
        
        // ========== FILES ==========
        @JavascriptInterface
        fun pickFile() {
            Log.d(TAG, "Pick file button clicked from JavaScript")
            runOnUiThread {
                pickFile()
            }
        }
        
        @JavascriptInterface
        fun saveFile(fileName: String, content: String) {
            Log.d(TAG, "Save file button clicked from JavaScript")
            runOnUiThread {
                saveFile(fileName, content)
            }
        }
        
        // ========== LOCATION ==========
        @JavascriptInterface
        fun getLocation() {
            Log.d(TAG, "Get location button clicked from JavaScript")
            runOnUiThread {
                getCurrentLocation()
            }
        }
        
        @JavascriptInterface
        fun startGPS() {
            Log.d(TAG, "Start GPS button clicked from JavaScript")
            runOnUiThread {
                getCurrentLocation()
            }
        }
        
        // ========== DEVICE INFO ==========
        @JavascriptInterface
        fun getBatteryLevel(): Int {
            Log.d(TAG, "Get battery level from JavaScript")
            return getBatteryLevel()
        }
        
        @JavascriptInterface
        fun getNetworkInfo(): String {
            Log.d(TAG, "Get network info from JavaScript")
            return getNetworkInfo()
        }
        
        @JavascriptInterface
        fun getDeviceInfo(): String {
            Log.d(TAG, "Get device info from JavaScript")
            return getDeviceInfo()
        }
        
        // ========== AUDIO ==========
        @JavascriptInterface
        fun recordAudio() {
            Log.d(TAG, "Record audio button clicked from JavaScript")
            runOnUiThread {
                if (isRecording) {
                    stopAudioRecording()
                } else {
                    startAudioRecording()
                }
            }
        }
        
        @JavascriptInterface
        fun playSound(soundType: String) {
            Log.d(TAG, "Play sound button clicked from JavaScript: $soundType")
            runOnUiThread {
                playSound()
            }
        }
        
        // ========== VIBRATION AND NOTIFICATIONS ==========
        @JavascriptInterface
        fun vibrate(duration: Long) {
            Log.d(TAG, "Vibrate button clicked from JavaScript: ${duration}ms")
            runOnUiThread {
                vibrateDevice(duration)
            }
        }
        
        @JavascriptInterface
        fun showNotification(title: String, message: String) {
            Log.d(TAG, "Show notification from JavaScript: $title - $message")
            runOnUiThread {
                showNotification(title, message)
            }
        }
        
        @JavascriptInterface
        fun showToast(message: String) {
            Log.d(TAG, "Show toast from JavaScript: $message")
            showToast(message)
        }
        
        // ========== COMMUNICATION ==========
        @JavascriptInterface
        fun sendSMS(phoneNumber: String, message: String) {
            Log.d(TAG, "Send SMS from JavaScript to: $phoneNumber")
            runOnUiThread {
                sendSms(phoneNumber, message)
            }
        }
        
        @JavascriptInterface
        fun makeCall(phoneNumber: String) {
            Log.d(TAG, "Make call from JavaScript to: $phoneNumber")
            runOnUiThread {
                makePhoneCall(phoneNumber)
            }
        }
        
        @JavascriptInterface
        fun sendEmail(recipient: String, subject: String, body: String) {
            Log.d(TAG, "Send email from JavaScript to: $recipient")
            runOnUiThread {
                sendEmail(recipient, subject, body)
            }
        }
        
        // ========== SYSTEM ==========
        @JavascriptInterface
        fun shareContent(content: String, mimeType: String) {
            Log.d(TAG, "Share content from JavaScript: $content")
            runOnUiThread {
                shareContent(content)
            }
        }
        
        @JavascriptInterface
        fun openBrowser(url: String) {
            Log.d(TAG, "Open browser from JavaScript: $url")
            runOnUiThread {
                openBrowser(url)
            }
        }
        
        @JavascriptInterface
        fun closeApp() {
            Log.d(TAG, "Close app from JavaScript")
            runOnUiThread {
                finish()
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
