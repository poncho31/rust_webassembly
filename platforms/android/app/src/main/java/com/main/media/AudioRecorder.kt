package com.main.media

import android.content.Context
import android.media.MediaRecorder
import android.os.Build
import android.util.Log
import android.widget.Toast
import com.main.utils.FileUtils
import java.io.File

class AudioRecorder(private val context: Context) {
    
    private var mediaRecorder: MediaRecorder? = null
    private var outputFile: File? = null
    var isRecording = false
        private set
    
    fun startRecording(): Boolean {
        try {
            outputFile = FileUtils.createAudioFile(context)
            
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
            
            isRecording = true
            Log.d("rust_webassembly_android", "Recording started: ${outputFile!!.absolutePath}")
            
            showToast("🎤 Enregistrement démarré")
            return true
            
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Recording failed: ${e.message}")
            showToast("Erreur d'enregistrement: ${e.message}")
            return false
        }
    }
    
    fun stopRecording(): String {
        Log.d("rust_webassembly_android", "Stopping audio recording")
        return try {
            mediaRecorder?.apply {
                stop()
                release()
            }
            mediaRecorder = null
            isRecording = false
            val filePath = outputFile?.absolutePath ?: ""
            Log.d("rust_webassembly_android", "Recording stopped: $filePath")
            
            // Copier le fichier vers le répertoire public pour qu'il soit accessible dans l'app musique
            if (filePath.isNotEmpty()) {
                val sourceFile = File(filePath)
                if (sourceFile.exists()) {
                    try {
                        val publicFilePath = FileUtils.copyAudioToPublicDirectory(context, sourceFile)
                        if (publicFilePath != null) {
                            Log.d("rust_webassembly_android", "Audio copied to public directory: $publicFilePath")
                            showToast("🎵 Enregistrement sauvé dans Musique: ${sourceFile.name} (${FileUtils.formatFileSize(sourceFile.length())})")
                            return publicFilePath
                        } else {
                            // Si la copie échoue, garder le fichier dans le répertoire privé
                            Log.w("rust_webassembly_android", "Failed to copy to public directory, keeping in private directory")
                            showToast("🎵 Enregistrement sauvé: ${sourceFile.name} (${FileUtils.formatFileSize(sourceFile.length())})")
                        }
                    } catch (e: Exception) {
                        Log.e("rust_webassembly_android", "Error copying audio to public directory: ${e.message}")
                        // Garder le fichier dans le répertoire privé en cas d'erreur
                        showToast("🎵 Enregistrement sauvé: ${sourceFile.name}")
                    }
                }
            } else {
                showToast("⏹️ Enregistrement arrêté")
            }
            filePath
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Stop recording failed: ${e.message}")
            isRecording = false
            showToast("Erreur arrêt enregistrement: ${e.message}")
            ""
        }
    }
    
    fun toggleRecording(): String {
        return if (isRecording) {
            stopRecording()
        } else {
            if (startRecording()) {
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
