package com.webassembly.unified.permissions

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat

class PermissionManager(
    private val activity: AppCompatActivity,
    private val onPermissionResult: (String?, Boolean) -> Unit
) {
    
    // Variables pour mémoriser les actions en attente de permission
    var pendingAction: String? = null
    var pendingSmsNumber: String? = null
    var pendingSmsMessage: String? = null
    
    private val requestPermissionLauncher = activity.registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted: Boolean ->
        if (isGranted) {
            Log.d("WebAssemblyApp", "Permission granted - executing pending action: $pendingAction")
        } else {
            Log.d("WebAssemblyApp", "Permission denied")
            clearPendingActions()
        }
        onPermissionResult(pendingAction, isGranted)
    }
    
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
    
    fun checkPermission(permission: String): Boolean {
        val result = when (permission) {
            "camera" -> ContextCompat.checkSelfPermission(activity, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
            "microphone" -> ContextCompat.checkSelfPermission(activity, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
            "location" -> ContextCompat.checkSelfPermission(activity, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED
            "sms" -> ContextCompat.checkSelfPermission(activity, Manifest.permission.SEND_SMS) == PackageManager.PERMISSION_GRANTED
            "storage" -> {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    ContextCompat.checkSelfPermission(activity, Manifest.permission.READ_MEDIA_IMAGES) == PackageManager.PERMISSION_GRANTED
                } else {
                    ContextCompat.checkSelfPermission(activity, Manifest.permission.WRITE_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
                }
            }
            else -> false
        }
        
        Log.d("WebAssemblyApp", "Permission $permission: $result")
        return result
    }
    
    fun setPendingAction(action: String, smsNumber: String? = null, smsMessage: String? = null) {
        pendingAction = action
        pendingSmsNumber = smsNumber
        pendingSmsMessage = smsMessage
    }
    
    fun clearPendingActions() {
        pendingAction = null
        pendingSmsNumber = null
        pendingSmsMessage = null
    }
    
    fun hasCameraAndMicrophonePermissions(): Boolean {
        return checkPermission("camera") && checkPermission("microphone")
    }
}
