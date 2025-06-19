package com.rustwebassembly.app;

import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.Bundle;
import android.os.IBinder;
import android.util.Log;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import okhttp3.Call;
import okhttp3.Callback;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;

import java.io.IOException;

/**
 * Activité principale de l'application
 */
public class MainActivity extends AppCompatActivity {
    private static final String TAG = "MainActivity";
    private static final String SERVER_BASE_URL = "http://127.0.0.1:8080";
    
    private TextView statusTextView;
    private TextView responseTextView;
    private Button startServerButton;
    private Button stopServerButton;
    private Button testApiButton;
    
    private RustServerService serverService;
    private boolean isServiceBound = false;
    private OkHttpClient httpClient;

    private ServiceConnection serviceConnection = new ServiceConnection() {
        @Override
        public void onServiceConnected(ComponentName name, IBinder service) {
            Log.d(TAG, "Service connecté");
            isServiceBound = true;
            updateUI();
        }

        @Override
        public void onServiceDisconnected(ComponentName name) {
            Log.d(TAG, "Service déconnecté");
            isServiceBound = false;
            serverService = null;
            updateUI();
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        initializeViews();
        setupClickListeners();
        
        httpClient = new OkHttpClient();
        
        // Démarre le service automatiquement
        startServerService();
    }

    private void initializeViews() {
        statusTextView = findViewById(R.id.statusTextView);
        responseTextView = findViewById(R.id.responseTextView);
        startServerButton = findViewById(R.id.startServerButton);
        stopServerButton = findViewById(R.id.stopServerButton);
        testApiButton = findViewById(R.id.testApiButton);
    }

    private void setupClickListeners() {
        startServerButton.setOnClickListener(v -> startServerService());
        stopServerButton.setOnClickListener(v -> stopServerService());
        testApiButton.setOnClickListener(v -> testApi());
    }

    private void startServerService() {
        Intent serviceIntent = new Intent(this, RustServerService.class);
        startService(serviceIntent);
        bindService(serviceIntent, serviceConnection, Context.BIND_AUTO_CREATE);
        
        statusTextView.setText("Démarrage du serveur...");
        Log.d(TAG, "Demande de démarrage du service serveur");
    }

    private void stopServerService() {
        if (isServiceBound) {
            unbindService(serviceConnection);
            isServiceBound = false;
        }
        
        Intent serviceIntent = new Intent(this, RustServerService.class);
        stopService(serviceIntent);
        
        statusTextView.setText("Serveur arrêté");
        responseTextView.setText("");
        updateUI();
        
        Log.d(TAG, "Arrêt du service serveur");
    }

    private void testApi() {
        if (!isServiceBound) {
            showToast("Le serveur n'est pas démarré");
            return;
        }

        // Test de l'endpoint ping
        String url = SERVER_BASE_URL + "/ping";
        Request request = new Request.Builder()
                .url(url)
                .build();

        httpClient.newCall(request).enqueue(new Callback() {
            @Override
            public void onFailure(Call call, IOException e) {
                runOnUiThread(() -> {
                    String errorMsg = "Erreur de connexion: " + e.getMessage();
                    responseTextView.setText(errorMsg);
                    showToast(errorMsg);
                    Log.e(TAG, "Erreur API: " + e.getMessage());
                });
            }

            @Override
            public void onResponse(Call call, Response response) throws IOException {
                final String responseBody = response.body() != null ? response.body().string() : "Pas de réponse";
                final boolean isSuccess = response.isSuccessful();
                
                runOnUiThread(() -> {
                    if (isSuccess) {
                        responseTextView.setText("✅ Réponse du serveur:\n" + responseBody);
                        showToast("API fonctionne !");
                        Log.d(TAG, "Réponse API: " + responseBody);
                    } else {
                        String errorMsg = "Erreur HTTP " + response.code() + ": " + responseBody;
                        responseTextView.setText("❌ " + errorMsg);
                        showToast(errorMsg);
                        Log.e(TAG, "Erreur HTTP: " + response.code());
                    }
                });
            }
        });
    }

    private void updateUI() {
        runOnUiThread(() -> {
            if (isServiceBound) {
                statusTextView.setText("✅ Serveur Rust actif sur le port 8080");
                startServerButton.setEnabled(false);
                stopServerButton.setEnabled(true);
                testApiButton.setEnabled(true);
            } else {
                statusTextView.setText("❌ Serveur arrêté");
                startServerButton.setEnabled(true);
                stopServerButton.setEnabled(false);
                testApiButton.setEnabled(false);
            }
        });
    }

    private void showToast(String message) {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (isServiceBound) {
            unbindService(serviceConnection);
        }
    }
}
