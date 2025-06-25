package com.webassembly.unified.handlers

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.webassembly.unified.media.CameraHandler
import com.webassembly.unified.utils.FileUtils
import org.json.JSONObject
import java.io.File

class ActivityResultHandler(private val activity: AppCompatActivity) {
    
    companion object {
        const val PICK_IMAGE_REQUEST = 1
        const val CAMERA_CAPTURE_REQUEST = 2
        const val VIDEO_RECORD_REQUEST = 3
        const val PICK_FILE_REQUEST = 1001
    }
    
    fun handleActivityResult(requestCode: Int, resultCode: Int, data: Intent?, cameraHandler: CameraHandler) {
        Log.d("WebAssemblyApp", "onActivityResult: requestCode=$requestCode, resultCode=$resultCode")
        
        when (requestCode) {
            PICK_IMAGE_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        handleImageSelected(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "Image picking cancelled")
                    showToast("Sélection d'image annulée")
                }
            }
            CAMERA_CAPTURE_REQUEST -> {
                Log.d("WebAssemblyApp", "Camera capture result - resultCode: $resultCode, photoFile exists: ${cameraHandler.photoFile?.exists()}")
                
                if (resultCode == Activity.RESULT_OK) {
                    val filePath = cameraHandler.handlePhotoCaptured()
                    if (filePath != null) {
                        showToast("📸 Photo sauvée: ${File(filePath).name} (${FileUtils.formatFileSize(File(filePath).length())})")
                    } else {
                        showToast("❌ Erreur lors de la sauvegarde de la photo")
                    }
                } else {
                    Log.d("WebAssemblyApp", "Photo capture cancelled")
                    showToast("Prise de photo annulée")
                    cameraHandler.handlePhotoCaptured() // Nettoyer même en cas d'annulation
                }
            }
            VIDEO_RECORD_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        handleVideoRecorded(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "Video recording cancelled")
                    showToast("Enregistrement vidéo annulé")
                }
            }
            PICK_FILE_REQUEST -> {
                if (resultCode == Activity.RESULT_OK) {
                    data?.data?.let { uri ->
                        handleFileSelected(uri)
                    }
                } else {
                    Log.d("WebAssemblyApp", "File picking cancelled")
                    showToast("Sélection de fichier annulée")
                }
            }
        }
    }
    
    private fun handleImageSelected(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing selected image: $uri")
        try {
            val imageInfo = JSONObject().apply {
                put("uri", uri.toString())
                put("type", "image")
                put("source", "gallery")
            }
            
            Log.d("WebAssemblyApp", "Image info: $imageInfo")
            
            showToast("🖼️ Image sélectionnée depuis la galerie")
            processImageFile(uri)
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing selected image: ${e.message}")
            showToast("Erreur lors du traitement de l'image: ${e.message}")
        }
    }
    
    private fun handleVideoRecorded(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing recorded video: $uri")
        try {
            val videoInfo = JSONObject().apply {
                put("uri", uri.toString())
                put("type", "video")
                put("source", "camera")
            }
            
            Log.d("WebAssemblyApp", "Video info: $videoInfo")
            showToast("🎬 Vidéo enregistrée")
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing recorded video: ${e.message}")
            showToast("Erreur lors du traitement de la vidéo: ${e.message}")
        }
    }
    
    private fun handleFileSelected(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing selected file: $uri")
        try {
            val fileInfo = JSONObject().apply {
                put("uri", uri.toString())
                put("type", "file")
                put("source", "picker")
            }
            
            Log.d("WebAssemblyApp", "File info: $fileInfo")
            showToast("📁 Fichier sélectionné")
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error processing selected file: ${e.message}")
            showToast("Erreur lors du traitement du fichier: ${e.message}")
        }
    }
    
    private fun processImageFile(uri: Uri) {
        Log.d("WebAssemblyApp", "Processing image file in Kotlin: $uri")
        try {
            // Traitement spécifique des images en Kotlin
            // Peut inclure redimensionnement, compression, etc.
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Error in image processing: ${e.message}")
        }
    }
    
    private fun showToast(message: String) {
        activity.runOnUiThread {
            Toast.makeText(activity, message, Toast.LENGTH_SHORT).show()
        }
    }
}
