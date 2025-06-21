package com.webassembly.unified

import android.os.Bundle
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.JavascriptInterface
import android.app.Activity
import android.util.Log
import android.webkit.WebSettings

class MainActivity : Activity() {
    
    private lateinit var webView: WebView
    
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
        
        // Initialize Rust backend
        if (initRust()) {
            Log.i(TAG, "Rust backend initialized successfully")
        } else {
            Log.e(TAG, "Failed to initialize Rust backend")
        }
        
        // Create and configure WebView
        webView = WebView(this)
        setupWebView()
        setContentView(webView)
        
        // Pour cette démo, charger une page HTML simple intégrée
        loadLocalHtml()
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
    }    private fun loadLocalHtml() {
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
    }
    
    override fun onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
    }
}
