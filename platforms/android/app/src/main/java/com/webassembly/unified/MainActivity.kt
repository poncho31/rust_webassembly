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
        
        // Add JavaScript interface to communicate with Rust
        webView.addJavascriptInterface(WebAppInterface(), "Android")
    }
    
    private fun loadLocalHtml() {
        val htmlContent = """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>WebAssembly Unified Frontend</title>
            <style>
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    margin: 0;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    min-height: 100vh;
                }
                .container {
                    max-width: 600px;
                    margin: 0 auto;
                    text-align: center;
                }
                h1 {
                    font-size: 2.5rem;
                    margin-bottom: 1rem;
                }
                .status {
                    background: rgba(255,255,255,0.1);
                    padding: 20px;
                    border-radius: 10px;
                    margin: 20px 0;
                }
                button {
                    background: #4CAF50;
                    color: white;
                    border: none;
                    padding: 15px 30px;
                    font-size: 1rem;
                    border-radius: 5px;
                    cursor: pointer;
                    margin: 10px;
                }
                button:hover {
                    background: #45a049;
                }
                #output {
                    background: rgba(0,0,0,0.3);
                    padding: 15px;
                    border-radius: 5px;
                    margin-top: 20px;
                    text-align: left;
                    min-height: 100px;
                    max-height: 300px;
                    overflow-y: auto;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🚀 WebAssembly Unified</h1>
                <div class="status">
                    <h3>Application Android avec Backend Rust</h3>
                    <p>L'intégration fonctionne correctement !</p>
                </div>
                
                <button onclick="testRustCommunication()">Tester Communication Rust</button>
                <button onclick="sendMessage()">Envoyer Message</button>
                <button onclick="clearOutput()">Effacer</button>
                
                <div id="output"></div>
            </div>
            
            <script>
                function log(message) {
                    const output = document.getElementById('output');
                    const timestamp = new Date().toLocaleTimeString();
                    output.innerHTML += '<div>[' + timestamp + '] ' + message + '</div>';
                    output.scrollTop = output.scrollHeight;
                    
                    // Aussi logger via Android
                    if (window.Android) {
                        window.Android.log(message);
                    }
                }
                
                function testRustCommunication() {
                    log('🔄 Test de communication avec Rust...');
                    
                    if (window.Android) {
                        const testMessage = JSON.stringify({
                            action: 'test',
                            data: 'Hello from JavaScript!',
                            timestamp: Date.now()
                        });
                        
                        try {
                            const response = window.Android.sendMessage(testMessage);
                            log('✅ Réponse de Rust: ' + response);
                        } catch (error) {
                            log('❌ Erreur: ' + error.toString());
                        }
                    } else {
                        log('❌ Interface Android non disponible');
                    }
                }
                
                function sendMessage() {
                    const message = prompt('Entrez votre message:');
                    if (message) {
                        log('📤 Envoi: ' + message);
                        
                        if (window.Android) {
                            const jsonMessage = JSON.stringify({
                                action: 'user_message',
                                message: message,
                                timestamp: Date.now()
                            });
                            
                            try {
                                const response = window.Android.sendMessage(jsonMessage);
                                log('📥 Réponse: ' + response);
                            } catch (error) {
                                log('❌ Erreur: ' + error.toString());
                            }
                        }
                    }
                }
                
                function clearOutput() {
                    document.getElementById('output').innerHTML = '';
                    log('🧹 Journal effacé');
                }
                
                // Test initial au chargement
                window.onload = function() {
                    log('🎉 Application chargée avec succès!');
                    setTimeout(testRustCommunication, 1000);
                };
            </script>
        </body>
        </html>
        """.trimIndent()
        
        webView.loadDataWithBaseURL(null, htmlContent, "text/html", "UTF-8", null)
        Log.i(TAG, "Local HTML content loaded")
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
