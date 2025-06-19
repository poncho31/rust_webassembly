package com.rustwebassembly.app;

import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.util.Log;
import androidx.annotation.Nullable;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Service Android qui gère le serveur Rust en arrière-plan
 */
public class RustServerService extends Service {
    private static final String TAG = "RustServerService";
    private static final String SERVER_BINARY = "librust_server.so";
    private static final int SERVER_PORT = 8080;
    
    private java.lang.Process serverProcess;
    private boolean isServerRunning = false;

    // Chargement de la bibliothèque native Rust
    static {
        try {
            System.loadLibrary("rust_server");
            Log.d(TAG, "Bibliothèque Rust chargée avec succès");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Erreur lors du chargement de la bibliothèque Rust: " + e.getMessage());
        }
    }

    @Nullable
    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        Log.d(TAG, "Démarrage du service serveur Rust");
        
        if (!isServerRunning) {
            startRustServer();
        }
        
        return START_STICKY; // Redémarre le service si tué
    }

    private void startRustServer() {
        try {
            // Copie le binaire depuis les assets vers un répertoire exécutable
            File serverFile = extractServerBinary();
            
            if (serverFile != null && serverFile.exists()) {
                // Démarre le serveur Rust
                ProcessBuilder pb = new ProcessBuilder(serverFile.getAbsolutePath());
                pb.environment().put("RUST_LOG", "info");
                pb.environment().put("SERVER_PORT", String.valueOf(SERVER_PORT));
                
                serverProcess = pb.start();
                isServerRunning = true;
                
                Log.i(TAG, "Serveur Rust démarré sur le port " + SERVER_PORT);
                
                // Surveille le processus en arrière-plan
                monitorServerProcess();
            } else {
                Log.e(TAG, "Impossible de trouver le binaire du serveur");
            }
        } catch (IOException e) {
            Log.e(TAG, "Erreur lors du démarrage du serveur: " + e.getMessage());
        }
    }

    private File extractServerBinary() {
        try {
            File filesDir = getFilesDir();
            File serverFile = new File(filesDir, "rust_server");
            
            // Copie depuis les assets si pas encore fait
            if (!serverFile.exists()) {
                InputStream inputStream = getAssets().open(SERVER_BINARY);
                FileOutputStream outputStream = new FileOutputStream(serverFile);
                
                byte[] buffer = new byte[1024];
                int length;
                while ((length = inputStream.read(buffer)) > 0) {
                    outputStream.write(buffer, 0, length);
                }
                
                inputStream.close();
                outputStream.close();
                
                // Rend le fichier exécutable
                serverFile.setExecutable(true);
                
                Log.d(TAG, "Binaire serveur extrait vers: " + serverFile.getAbsolutePath());
            }
            
            return serverFile;
        } catch (IOException e) {
            Log.e(TAG, "Erreur lors de l'extraction du binaire: " + e.getMessage());
            return null;
        }
    }

    private void monitorServerProcess() {
        new Thread(() -> {
            try {
                if (serverProcess != null) {
                    int exitCode = serverProcess.waitFor();
                    Log.w(TAG, "Le serveur s'est arrêté avec le code: " + exitCode);
                    isServerRunning = false;
                    
                    // Redémarre automatiquement si arrêt inattendu
                    if (exitCode != 0) {
                        Log.i(TAG, "Redémarrage automatique du serveur...");
                        startRustServer();
                    }
                }
            } catch (InterruptedException e) {
                Log.e(TAG, "Surveillance du serveur interrompue: " + e.getMessage());
            }
        }).start();
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        stopRustServer();
    }

    private void stopRustServer() {
        if (serverProcess != null && isServerRunning) {
            Log.d(TAG, "Arrêt du serveur Rust");
            serverProcess.destroy();
            
            try {
                serverProcess.waitFor(5000, java.util.concurrent.TimeUnit.MILLISECONDS);
            } catch (InterruptedException e) {
                Log.w(TAG, "Timeout lors de l'arrêt du serveur");
                serverProcess.destroyForcibly();
            }
            
            isServerRunning = false;
            Log.i(TAG, "Serveur Rust arrêté");
        }
    }

    public boolean isServerRunning() {
        return isServerRunning;
    }

    public int getServerPort() {
        return SERVER_PORT;
    }
}
