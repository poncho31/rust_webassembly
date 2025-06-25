package com.webassembly.unified.utils

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.BatteryManager
import android.os.Build
import android.util.Log
import org.json.JSONObject

object DeviceUtils {
    
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
    }
    
    fun getBatteryLevel(context: Context): Int {
        Log.d("WebAssemblyApp", "Getting battery level")
        return try {
            val batteryManager = context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager
            val batteryLevel = batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
            Log.d("WebAssemblyApp", "Battery level: $batteryLevel%")
            batteryLevel
        } catch (e: Exception) {
            Log.e("WebAssemblyApp", "Failed to get battery level: ${e.message}")
            -1
        }
    }
    
    fun getNetworkInfo(context: Context): String {
        Log.d("WebAssemblyApp", "Getting network info")
        return try {
            val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
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
            "{\"connected\": false, \"type\": \"Error\", \"error\": \"${e.message}\"}"
        }
    }
}
