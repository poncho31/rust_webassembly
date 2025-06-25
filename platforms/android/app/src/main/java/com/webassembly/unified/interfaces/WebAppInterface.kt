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
        permissionManager.requestPermission(permission)
    }

    @JavascriptInterface
    fun checkPermission(permission: String): Boolean {
        return permissionManager.checkPermission(permission)
    }

    // Méthodes audio
    @JavascriptInterface
    fun startRecording() {
        Log.d("WebAssemblyApp", "Toggle audio recording - currently recording: ${audioRecorder.isRecording}")
        
        if (permissionManager.checkPermission("microphone")) {
            audioRecorder.toggleRecording()
        } else {
            Log.d("WebAssemblyApp", "Requesting microphone permission for recording")
            permissionManager.setPendingAction("startRecording")
            permissionManager.requestPermission("microphone")
        }
    }

    @JavascriptInterface
    fun stopRecording(): String {
        Log.d("WebAssemblyApp", "Stop recording called from JS")
        return audioRecorder.stopRecording()
    }

    @JavascriptInterface
    fun recordAudio() {
        Log.d("WebAssemblyApp", "Record audio called - toggle mode")
        startRecording()
    }

    @JavascriptInterface
    fun isRecording(): Boolean {
        Log.d("WebAssemblyApp", "Checking recording status: ${audioRecorder.isRecording}")
        return audioRecorder.isRecording
    }

    // Méthodes vidéo
    @JavascriptInterface
    fun recordVideoBackground() {
        Log.d("WebAssemblyApp", "Toggle video recording - currently recording: ${videoRecorder.isVideoRecording}")
        
        if (permissionManager.hasCameraAndMicrophonePermissions()) {
            videoRecorder.toggleVideoRecording(surfaceHolder)
        } else {
            Log.d("WebAssemblyApp", "Missing permissions for video recording")
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
        Log.d("WebAssemblyApp", "Checking video recording status: ${videoRecorder.isVideoRecording}")
        return videoRecorder.isVideoRecording
    }

    @JavascriptInterface
    fun stopVideoRecording(): String {
        Log.d("WebAssemblyApp", "Stop video recording called from JS")
        return videoRecorder.stopVideoRecording()
    }

    // Méthodes caméra
    @JavascriptInterface
    fun takePhoto() {
        Log.d("WebAssemblyApp", "Taking photo")
        if (permissionManager.checkPermission("camera")) {
            executeCamera()
        } else {
            Log.d("WebAssemblyApp", "Requesting camera permission for photo")
            permissionManager.setPendingAction("takePhoto")
            permissionManager.requestPermission("camera")
        }
    }

    @JavascriptInterface
    fun recordVideo() {
        Log.d("WebAssemblyApp", "Recording video")
        if (permissionManager.checkPermission("camera")) {
            executeVideoRecording()
        } else {
            Log.d("WebAssemblyApp", "Requesting camera permission for video")
            permissionManager.setPendingAction("recordVideo")
            permissionManager.requestPermission("camera")
        }
    }

    @JavascriptInterface
    fun openCamera() {
        Log.d("WebAssemblyApp", "Opening camera")
        if (permissionManager.checkPermission("camera")) {
            try {
                cameraHandler.takePhoto()
            } catch (e: Exception) {
                showToast("Erreur appareil photo: ${e.message}")
            }
        } else {
            Log.e("WebAssemblyApp", "No permission for camera")
            showToast("Permission appareil photo requise")
        }
    }

    @JavascriptInterface
    fun openGallery() {
        Log.d("WebAssemblyApp", "Opening gallery")
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "image/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        activity.startActivityForResult(intent, 1)
    }

    @JavascriptInterface
    fun openFile() {
        Log.d("WebAssemblyApp", "Opening file picker")
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
            Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
            showToast("Erreur appareil photo: ${e.message}")
        }
    }

    fun executeVideoRecording() {
        try {
            cameraHandler.recordVideo()
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
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
        Log.d("WebAssemblyApp", "Vibrating for ${duration}ms")
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
        Log.d("WebAssemblyApp", "Sending SMS to $number: $message")
        if (permissionManager.checkPermission("sms")) {
            return executeSendSMS(number, message)
        } else {
            Log.d("WebAssemblyApp", "Requesting SMS permission")
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
            Log.d("WebAssemblyApp", "Requesting location permission")
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
            Log.d("WebAssemblyApp", "Location: $result")
            result
        } else {
            Log.d("WebAssemblyApp", "Location not available")
            "{\"error\": \"Location not available\"}"
        }
    }

    @JavascriptInterface
    fun saveFile(filename: String, content: String): Boolean {
        Log.d("WebAssemblyApp", "Saving file: $filename")
        return try {
            val file = File(activity.externalCacheDir, filename)
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
        return DeviceUtils.getDeviceInfo()
    }

    @JavascriptInterface
    fun showToast(message: String) {
        Log.d("WebAssemblyApp", "Showing toast: $message")
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
            vibrate(100)
            Log.d("WebAssemblyApp", "Sound played via vibration")
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Failed to play sound: ${e.message}")
        }
    }

    @JavascriptInterface
    fun showNotification(title: String, message: String) {
        Log.d("WebAssemblyApp", "Showing notification: $title - $message")
        activity.runOnUiThread {
            Toast.makeText(activity, "$title: $message", Toast.LENGTH_LONG).show()
        }
    }

    @JavascriptInterface
    fun makeCall(phoneNumber: String) {
        Log.d("WebAssemblyApp", "Making call to: $phoneNumber")
        try {
            val intent = Intent(Intent.ACTION_DIAL).apply {
                data = Uri.parse("tel:$phoneNumber")
            }
            activity.startActivity(intent)
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
            activity.startActivity(intent)
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
            activity.startActivity(Intent.createChooser(intent, "Partager via"))
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
            activity.startActivity(intent)
            Log.d("WebAssemblyApp", "Browser intent launched")
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Failed to open browser: ${e.message}")
        }
    }

    @JavascriptInterface
    fun closeApp() {
        Log.d("WebAssemblyApp", "Closing application")
        activity.runOnUiThread {
            activity.finish()
        }
    }
}
