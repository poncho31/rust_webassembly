package com.main.utils

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.util.Log
import java.io.File
import java.io.FileInputStream
import java.io.OutputStream
import java.text.SimpleDateFormat
import java.util.*

object FileUtils {
    
    fun createImageFile(context: Context): File {
        val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
        val storageDir = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
        return File.createTempFile(
            "JPEG_${timeStamp}_",
            ".jpg",
            storageDir
        )
    }
    
    fun createAudioFile(context: Context): File {
        val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
        val fileName = "REC_${timeStamp}.3gp"
        
        val audioDir = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            context.getExternalFilesDir(Environment.DIRECTORY_MUSIC)
        } else {
            val musicDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MUSIC)
            val appDir = File(musicDir, "rust_webassembly_android")
            appDir.mkdirs()
            appDir
        }
        
        return File(audioDir, fileName)
    }
    
    fun createVideoFile(context: Context): File {
        val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
        val fileName = "VID_${timeStamp}.mp4"
        
        val videoDir = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            context.getExternalFilesDir(Environment.DIRECTORY_MOVIES)
        } else {
            val moviesDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MOVIES)
            val appDir = File(moviesDir, "rust_webassembly_android")
            appDir.mkdirs()
            appDir
        }
        
        return File(videoDir, fileName)
    }
    
    fun formatFileSize(bytes: Long): String {
        val kb = bytes / 1024.0
        val mb = kb / 1024.0
        val gb = mb / 1024.0
        
        return when {
            gb >= 1 -> String.format("%.2f GB", gb)
            mb >= 1 -> String.format("%.2f MB", mb)
            kb >= 1 -> String.format("%.2f KB", kb)
            else -> "$bytes B"
        }
    }
    
    fun copyAudioToPublicDirectory(context: Context, sourceFile: File): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val contentValues = ContentValues().apply {
                    put(MediaStore.Audio.Media.DISPLAY_NAME, sourceFile.name)
                    put(MediaStore.Audio.Media.MIME_TYPE, "audio/3gpp")
                    put(MediaStore.Audio.Media.RELATIVE_PATH, Environment.DIRECTORY_MUSIC + "/rust_webassembly_android")
                }
                
                val uri = context.contentResolver.insert(MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, contentValues)
                uri?.let { targetUri ->
                    context.contentResolver.openOutputStream(targetUri)?.use { outputStream ->
                        FileInputStream(sourceFile).use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    targetUri.toString()
                }
            } else {
                val publicDir = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MUSIC), "rust_webassembly_android")
                publicDir.mkdirs()
                val publicFile = File(publicDir, sourceFile.name)
                sourceFile.copyTo(publicFile, overwrite = true)
                publicFile.absolutePath
            }
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Error copying audio to public directory: ${e.message}")
            null
        }
    }
    
    fun copyVideoToPublicDirectory(context: Context, sourceFile: File): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val contentValues = ContentValues().apply {
                    put(MediaStore.Video.Media.DISPLAY_NAME, sourceFile.name)
                    put(MediaStore.Video.Media.MIME_TYPE, "video/mp4")
                    put(MediaStore.Video.Media.RELATIVE_PATH, Environment.DIRECTORY_MOVIES + "/rust_webassembly_android")
                }
                
                val uri = context.contentResolver.insert(MediaStore.Video.Media.EXTERNAL_CONTENT_URI, contentValues)
                uri?.let { targetUri ->
                    context.contentResolver.openOutputStream(targetUri)?.use { outputStream ->
                        FileInputStream(sourceFile).use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    targetUri.toString()
                }
            } else {
                val publicDir = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MOVIES), "rust_webassembly_android")
                publicDir.mkdirs()
                val publicFile = File(publicDir, sourceFile.name)
                sourceFile.copyTo(publicFile, overwrite = true)
                publicFile.absolutePath
            }
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Error copying video to public directory: ${e.message}")
            null
        }
    }
    
    fun copyPhotoToPublicDirectory(context: Context, sourceFile: File): String? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val contentValues = ContentValues().apply {
                    put(MediaStore.Images.Media.DISPLAY_NAME, sourceFile.name)
                    put(MediaStore.Images.Media.MIME_TYPE, "image/jpeg")
                    put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/rust_webassembly_android")
                }
                
                val uri = context.contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)
                uri?.let { targetUri ->
                    context.contentResolver.openOutputStream(targetUri)?.use { outputStream ->
                        FileInputStream(sourceFile).use { inputStream ->
                            inputStream.copyTo(outputStream)
                        }
                    }
                    targetUri.toString()
                }
            } else {
                val publicDir = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES), "rust_webassembly_android")
                publicDir.mkdirs()
                val publicFile = File(publicDir, sourceFile.name)
                sourceFile.copyTo(publicFile, overwrite = true)
                publicFile.absolutePath
            }
        } catch (e: Exception) {
            Log.e("rust_webassembly_android", "Error copying photo to public directory: ${e.message}")
            null
        }
    }
}
