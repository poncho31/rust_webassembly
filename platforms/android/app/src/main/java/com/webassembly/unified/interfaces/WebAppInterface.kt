package com.webassembly.unified.interfaces

import android.content.Context
import android.content.Intent
import android.location.LocationManager
import android.net.Uri
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.telephony.SmsManager
import android.util.Log
import android.webkit.JavascriptInterface
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.webassembly.unified.media.AudioRecorder
import com.webassembly.unified.media.CameraHandler
import com.webassembly.unified.media.VideoRecorder
import com.webassembly.unified.permissions.PermissionManager
import com.webassembly.unified.utils.DeviceUtils
import com.webassembly.unified.utils.FileUtils
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream

class WebAppInterface(
    private val activity: AppCompatActivity,
    private val permissionManager: PermissionManager,
    private val audioRecorder: AudioRecorder,
    private val videoRecorder: VideoRecorder,
    private val cameraHandler: CameraHandler,
    private val surfaceHolder: android.view.SurfaceHolder?
) {

    @JavascriptInterface
    fun log(message: String) {
        Log.d("rust_webassembly_android", "JS Log: $message")
    }

    @JavascriptInterface
    fun error(message: String) {
        Log.e("rust_webassembly_android", "JS Error: $message")
    }

    @JavascriptInterface
    fun warn(message: String) {
        Log.w("rust_webassembly_android", "JS Warning: $message")
    }

    @JavascriptInterface
    fun info(message: String) {
        Log.i("rust_webassembly_android", "JS Info: $message")
    }

    @JavascriptInterface
    fun requestPermission(permission: String) {
        permissionManager.requestPermission(permission)
    }

    @JavascriptInterface
    fun checkPermission(permission: String): Boolean {
        return permissionManager.checkPermission(permission)
    }

    // Méthodes audio
    @JavascriptInterface
    fun startRecording() {
        Log.d("rust_webassembly_android", "Toggle audio recording - currently recording: ${audioRecorder.isRecording}")
        
        if (permissionManager.checkPermission("microphone")) {
            audioRecorder.toggleRecording()
        } else {
            Log.d("rust_webassembly_android", "Requesting microphone permission for recording")
            permissionManager.setPendingAction("startRecording")
            permissionManager.requestPermission("microphone")
        }
    }

    @JavascriptInterface
    fun stopRecording(): String {
        Log.d("rust_webassembly_android", "Stop recording called from JS")
        return audioRecorder.stopRecording()
    }

    @JavascriptInterface
    fun recordAudio() {
        Log.d("rust_webassembly_android", "Record audio called - toggle mode")
        startRecording()
    }

    @JavascriptInterface
    fun isRecording(): Boolean {
        Log.d("rust_webassembly_android", "Checking recording status: ${audioRecorder.isRecording}")
        return audioRecorder.isRecording
    }

    // Méthodes vidéo
    @JavascriptInterface
    fun recordVideoBackground() {
        Log.d("rust_webassembly_android", "Toggle video recording - currently recording: ${videoRecorder.isVideoRecording}")
        
        if (permissionManager.hasCameraAndMicrophonePermissions()) {
            videoRecorder.toggleVideoRecording(surfaceHolder)
        } else {
            Log.d("rust_webassembly_android", "Missing permissions for video recording")
            permissionManager.setPendingAction("recordVideoBackground")
            
            if (!permissionManager.checkPermission("camera")) {
                permissionManager.requestPermission("camera")
            } else if (!permissionManager.checkPermission("microphone")) {
                permissionManager.requestPermission("microphone")
            }
        }
    }

    @JavascriptInterface
    fun isVideoRecording(): Boolean {
        Log.d("rust_webassembly_android", "Checking video recording status: ${videoRecorder.isVideoRecording}")
        return videoRecorder.isVideoRecording
    }

    @JavascriptInterface
    fun stopVideoRecording(): String {
        Log.d("rust_webassembly_android", "Stop video recording called from JS")
        return videoRecorder.stopVideoRecording()
    }

    // Méthodes caméra
    @JavascriptInterface
    fun takePhoto() {
        Log.d("rust_webassembly_android", "Taking photo")
        if (permissionManager.checkPermission("camera")) {
            executeCamera()
        } else {
            Log.d("rust_webassembly_android", "Requesting camera permission for photo")
            permissionManager.setPendingAction("takePhoto")
            permissionManager.requestPermission("camera")
        }
    }

    @JavascriptInterface
    fun recordVideo() {
        Log.d("rust_webassembly_android", "Recording video")
        if (permissionManager.checkPermission("camera")) {
            executeVideoRecording()
        } else {
            Log.d("rust_webassembly_android", "Requesting camera permission for video")
            permissionManager.setPendingAction("recordVideo")
            permissionManager.requestPermission("camera")
        }
    }

    @JavascriptInterface
    fun openCamera() {
        Log.d("rust_webassembly_android", "Opening camera")
        if (permissionManager.checkPermission("camera")) {
            try {
                cameraHandler.takePhoto()
            } catch (e: Exception) {
                showToast("Erreur appareil photo: ${e.message}")
            }
        } else {
            Log.e("rust_webassembly_android", "No permission for camera")
            showToast("Permission appareil photo requise")
        }
    }

    @JavascriptInterface
    fun openGallery() {
        Log.d("rust_webassembly_android", "Opening gallery")
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "image/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        activity.startActivityForResult(intent, 1)
    }

    @JavascriptInterface
    fun openFile() {
        Log.d("rust_webassembly_android", "Opening file picker")
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "*/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        activity.startActivityForResult(intent, 1001)
    }

    // Méthodes exécution
    fun executeCamera() {
        try {
            cameraHandler.takePhoto()
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Camera failed: ${e.message}")
            showToast("Erreur appareil photo: ${e.message}")
        }
    }

    fun executeVideoRecording() {
        try {
            cameraHandler.recordVideo()
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Video recording failed: ${e.message}")
            showToast("Erreur enregistrement vidéo: ${e.message}")
        }
    }

    fun executeAudioRecording() {
        audioRecorder.startRecording()
    }

    fun executeVideoBackgroundRecording() {
        videoRecorder.startVideoRecording(surfaceHolder)
    }

    // Autres méthodes
    @JavascriptInterface
    fun vibrate(duration: Long) {
        Log.d("rust_webassembly_android", "Vibrating for ${duration}ms")
        val vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            val vibratorManager = activity.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
            vibratorManager.defaultVibrator
        } else {
            @Suppress("DEPRECATION")
            activity.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
        }
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(duration)
        }
    }

    @JavascriptInterface
    fun sendSMS(number: String, message: String): Boolean {
        Log.d("rust_webassembly_android", "Sending SMS to $number: $message")
        if (permissionManager.checkPermission("sms")) {
            return executeSendSMS(number, message)
        } else {
            Log.d("rust_webassembly_android", "Requesting SMS permission")
            permissionManager.setPendingAction("sendSMS", number, message)
            permissionManager.requestPermission("sms")
            return false
        }
    }

    fun executeSendSMS(number: String, message: String): Boolean {
        return try {
            val smsManager = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                activity.getSystemService(SmsManager::class.java)
            } else {
                @Suppress("DEPRECATION")
                SmsManager.getDefault()
            }
            smsManager.sendTextMessage(number, null, message, null, null)
            Log.d("rust_webassembly_android", "SMS sent successfully")
            true
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "SMS failed: ${e.message}")
            false
        }
    }

    @JavascriptInterface
    fun getLocation(): String {
        Log.d("rust_webassembly_android", "Getting location")
        if (permissionManager.checkPermission("location")) {
            val result = executeGetLocation()
            
            // Afficher le résultat dans un popup
            activity.runOnUiThread {
                try {
                    val locationJson = JSONObject(result)
                    if (locationJson.has("latitude") && locationJson.has("longitude")) {
                        val lat = locationJson.getDouble("latitude")
                        val lng = locationJson.getDouble("longitude")
                        Toast.makeText(activity, "Position: $lat, $lng", Toast.LENGTH_LONG).show()
                    } else {
                        Toast.makeText(activity, "Position non disponible", Toast.LENGTH_SHORT).show()
                    }
                } catch (e: Exception) {
                    Toast.makeText(activity, "Erreur de position: ${e.message}", Toast.LENGTH_SHORT).show()
                }
            }
            
            return result
        } else {
            Log.d("rust_webassembly_android", "Requesting location permission")
            permissionManager.setPendingAction("getLocation")
            permissionManager.requestPermission("location")
            return "{\"error\": \"Permission not granted\"}"
        }
    }

    fun executeGetLocation(): String {
        val locationManager = activity.getSystemService(Context.LOCATION_SERVICE) as LocationManager
        val location = locationManager.getLastKnownLocation(LocationManager.GPS_PROVIDER)
        
        return if (location != null) {
            val result = "{\"latitude\": ${location.latitude}, \"longitude\": ${location.longitude}}"
            Log.d("rust_webassembly_android", "Location: $result")
            result
        } else {
            Log.d("rust_webassembly_android", "Location not available")
            "{\"error\": \"Location not available\"}"
        }
    }

    @JavascriptInterface
    fun saveFile(filename: String, content: String): Boolean {
        Log.d("rust_webassembly_android", "Saving file: $filename")
        return try {
            val file = File(activity.externalCacheDir, filename)
            FileOutputStream(file).use { 
                it.write(content.toByteArray()) 
            }
            Log.d("rust_webassembly_android", "File saved: ${file.absolutePath}")
            true
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Save file failed: ${e.message}")
            false
        }
    }

    @JavascriptInterface
    fun getDeviceInfo(): String {
        return DeviceUtils.getDeviceInfo()
    }

    @JavascriptInterface
    fun showToast(message: String) {
        Log.d("rust_webassembly_android", "Showing toast: $message")
        activity.runOnUiThread {
            Toast.makeText(activity, message, Toast.LENGTH_SHORT).show()
        }
    }

    @JavascriptInterface
    fun getBatteryLevel(): Int {
        return DeviceUtils.getBatteryLevel(activity)
    }

    @JavascriptInterface
    fun getNetworkInfo(): String {
        return DeviceUtils.getNetworkInfo(activity)
    }

    @JavascriptInterface
    fun pickImage() {
        Log.d("rust_webassembly_android", "Pick image called")
        openGallery()
    }

    @JavascriptInterface
    fun pickFile() {
        Log.d("rust_webassembly_android", "Pick file called") 
        openFile()
    }

    @JavascriptInterface
    fun startGPS() {
        Log.d("rust_webassembly_android", "Start GPS called")
        getLocation()
    }

    @JavascriptInterface
    fun playSound(soundType: String) {
        Log.d("rust_webassembly_android", "Playing sound: $soundType")
        try {
            vibrate(100)
            Log.d("rust_webassembly_android", "Sound played via vibration")
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Failed to play sound: ${e.message}")
        }
    }

    @JavascriptInterface
    fun showNotification(title: String, message: String) {
        Log.d("rust_webassembly_android", "Showing notification: $title - $message")
        activity.runOnUiThread {
            Toast.makeText(activity, "$title: $message", Toast.LENGTH_LONG).show()
        }
    }

    @JavascriptInterface
    fun makeCall(phoneNumber: String) {
        Log.d("rust_webassembly_android", "Making call to: $phoneNumber")
        try {
            val intent = Intent(Intent.ACTION_DIAL).apply {
                data = Uri.parse("tel:$phoneNumber")
            }
            activity.startActivity(intent)
            Log.d("rust_webassembly_android", "Call intent launched")
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Failed to make call: ${e.message}")
        }
    }

    @JavascriptInterface
    fun sendEmail(email: String, subject: String, body: String) {
        Log.d("rust_webassembly_android", "Sending email to: $email")
        try {
            val intent = Intent(Intent.ACTION_SENDTO).apply {
                data = Uri.parse("mailto:")
                putExtra(Intent.EXTRA_EMAIL, arrayOf(email))
                putExtra(Intent.EXTRA_SUBJECT, subject)
                putExtra(Intent.EXTRA_TEXT, body)
            }
            activity.startActivity(intent)
            Log.d("rust_webassembly_android", "Email intent launched")
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Failed to send email: ${e.message}")
        }
    }

    @JavascriptInterface
    fun shareContent(content: String, mimeType: String) {
        Log.d("rust_webassembly_android", "Sharing content: $content")
        try {
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = mimeType
                putExtra(Intent.EXTRA_TEXT, content)
            }
            activity.startActivity(Intent.createChooser(intent, "Partager via"))
            Log.d("rust_webassembly_android", "Share intent launched")
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Failed to share content: ${e.message}")
        }
    }

    @JavascriptInterface
    fun openBrowser(url: String) {
        Log.d("rust_webassembly_android", "Opening browser with URL: $url")
        try {
            val intent = Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse(url)
            }
            activity.startActivity(intent)
            Log.d("rust_webassembly_android", "Browser intent launched")
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Failed to open browser: ${e.message}")
        }
    }

    @JavascriptInterface
    fun closeApp() {
        Log.d("rust_webassembly_android", "Closing application")
        activity.runOnUiThread {
            activity.finish()
        }
    }
}
