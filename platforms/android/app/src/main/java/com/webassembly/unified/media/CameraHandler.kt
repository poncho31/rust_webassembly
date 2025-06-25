package com.webassembly.unified.media

import android.content.Context
import android.content.Intent
import android.hardware.Camera
import android.provider.MediaStore
import android.util.Log
import android.view.SurfaceHolder
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.FileProvider
import com.webassembly.unified.utils.FileUtils
import java.io.File

class CameraHandler(private val activity: AppCompatActivity) {
    
    companion object {
        const val CAMERA_CAPTURE_REQUEST = 2
    }
    
    var photoFile: File? = null
        private set
    var currentPhotoPath: String? = null
        private set
    
    fun takePhoto() {
        try {
            photoFile = FileUtils.createImageFile(activity)
            currentPhotoPath = photoFile?.absolutePath
            
            val photoURI = FileProvider.getUriForFile(
                activity,
                "com.webassembly.unified.fileprovider",
                photoFile!!
            )
            
            Log.d("WebAssemblyApp", "Photo will be saved to: $currentPhotoPath")
            
            val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
                putExtra(MediaStore.EXTRA_OUTPUT, photoURI)
                addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            
            if (intent.resolveActivity(activity.packageManager) != null) {
                activity.startActivityForResult(intent, CAMERA_CAPTURE_REQUEST)
            } else {
                Log.e("WebAssemblyApp", "No camera app available")
                throw Exception("Aucune application appareil photo disponible")
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Camera failed: ${e.message}")
            throw e
        }
    }
    
    fun recordVideo() {
        try {
            val intent = Intent(MediaStore.ACTION_VIDEO_CAPTURE)
            activity.startActivityForResult(intent, 3)
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
            throw e
        }
    }
    
    fun handlePhotoCaptured(): String? {
        Log.d("WebAssemblyApp", "Processing captured photo: $currentPhotoPath")
        return try {
            currentPhotoPath?.let { path ->
                val file = File(path)
                if (file.exists()) {
                    val publicFilePath = FileUtils.copyPhotoToPublicDirectory(activity, file)
                    if (publicFilePath != null) {
                        Log.d("WebAssemblyApp", "Photo copied to public directory: $publicFilePath")
                        showToast("📸 Photo sauvée dans Galerie: ${file.name} (${FileUtils.formatFileSize(file.length())})")
                        return publicFilePath
                    } else {
                        Log.w("WebAssemblyApp", "Failed to copy photo to public directory, keeping in private directory")
                        showToast("📸 Photo sauvée: ${file.name} (${FileUtils.formatFileSize(file.length())})")
                        return path
                    }
                } else {
                    Log.w("WebAssemblyApp", "Photo file does not exist: $path")
                    showToast("❌ Fichier photo introuvable")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing captured photo: ${e.message}")
            showToast("❌ Erreur lors de la sauvegarde de la photo: ${e.message}")
            null
        } finally {
            currentPhotoPath = null
            photoFile = null
        }
    }
    
    fun restoreState(savedPhotoPath: String?) {
        currentPhotoPath = savedPhotoPath
        currentPhotoPath?.let { path ->
            photoFile = File(path)
            if (!photoFile!!.exists()) {
                photoFile = null
                currentPhotoPath = null
            }
        }
    }
    
    fun saveState(): String? = currentPhotoPath
    
    private fun showToast(message: String) {
        activity.runOnUiThread {
            Toast.makeText(activity, message, Toast.LENGTH_SHORT).show()
        }
    }
}
