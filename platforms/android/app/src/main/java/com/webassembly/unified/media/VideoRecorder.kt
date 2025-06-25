package com.webassembly.unified.media

import android.content.Context
import android.hardware.Camera
import android.media.MediaRecorder
import android.os.Build
import android.util.Log
import android.view.SurfaceHolder
import android.widget.Toast
import com.webassembly.unified.utils.FileUtils
import java.io.File

class VideoRecorder(private val context: Context) {
    
    private var videoMediaRecorder: MediaRecorder? = null
    private var camera: Camera? = null
    private var videoOutputFile: File? = null
    var isVideoRecording = false
        private set
    
    fun startVideoRecording(surfaceHolder: SurfaceHolder?): Boolean {
        try {
            videoOutputFile = FileUtils.createVideoFile(context)
            
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
                throw e
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
            
            isVideoRecording = true
            Log.d("WebAssemblyApp", "Video recording started: ${videoOutputFile!!.absolutePath}")
            
            showToast("🎥 Enregistrement vidéo démarré")
            return true
            
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Video recording failed: ${e.message}")
            // Nettoyer en cas d'erreur
            videoMediaRecorder?.release()
            videoMediaRecorder = null
            camera?.release()
            camera = null
            isVideoRecording = false
            
            showToast("Erreur enregistrement vidéo: ${e.message}")
            return false
        }
    }
    
    fun stopVideoRecording(): String {
        Log.d("WebAssemblyApp", "Stopping video recording")
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
            
            isVideoRecording = false
            val filePath = videoOutputFile?.absolutePath ?: ""
            Log.d("WebAssemblyApp", "Video recording stopped: $filePath")
            
            // Copier le fichier vers le répertoire public pour qu'il soit accessible dans l'app galerie
            if (filePath.isNotEmpty()) {
                val sourceFile = File(filePath)
                if (sourceFile.exists()) {
                    try {
                        val publicFilePath = FileUtils.copyVideoToPublicDirectory(context, sourceFile)
                        if (publicFilePath != null) {
                            Log.d("WebAssemblyApp", "Video copied to public directory: $publicFilePath")
                            showToast("🎬 Vidéo sauvée dans Galerie: ${sourceFile.name} (${FileUtils.formatFileSize(sourceFile.length())})")
                            return publicFilePath
                        } else {
                            // Si la copie échoue, garder le fichier dans le répertoire privé
                            Log.w("WebAssemblyApp", "Failed to copy video to public directory, keeping in private directory")
                            showToast("🎬 Vidéo sauvée: ${sourceFile.name} (${FileUtils.formatFileSize(sourceFile.length())})")
                        }
                    } catch (e: Exception) {
                        Log.e("WebAssemblyApp", "Error copying video to public directory: ${e.message}")
                        // Garder le fichier dans le répertoire privé en cas d'erreur
                        showToast("🎬 Vidéo sauvée: ${sourceFile.name}")
                    }
                }
            } else {
                showToast("⏹️ Enregistrement vidéo arrêté")
            }
            filePath
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Stop video recording failed: ${e.message}")
            // Nettoyer en cas d'erreur
            videoMediaRecorder?.release()
            videoMediaRecorder = null
            camera?.release()
            camera = null
            isVideoRecording = false
            
            showToast("Erreur arrêt enregistrement vidéo: ${e.message}")
            ""
        }
    }
    
    fun toggleVideoRecording(surfaceHolder: SurfaceHolder?): String {
        return if (isVideoRecording) {
            stopVideoRecording()
        } else {
            if (startVideoRecording(surfaceHolder)) {
                ""
            } else {
                ""
            }
        }
    }
    
    private fun showToast(message: String) {
        if (context is android.app.Activity) {
            context.runOnUiThread {
                Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
            }
        }
    }
}
