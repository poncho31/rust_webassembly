use crate::refresh::{RefreshConfig, RefreshHandler};
use crate::client_tools::log;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;

/// Planificateur pour gérer plusieurs rafraîchissements automatiques
pub struct RefreshScheduler {
    handlers: HashMap<String, RefreshHandler>,
}

impl RefreshScheduler {
    /// Créer un nouveau planificateur
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Ajouter une configuration de rafraîchissement
    pub fn add_refresh(mut self, config: RefreshConfig) -> Self {
        let handler = RefreshHandler::new(config.clone());
        self.handlers.insert(config.id.clone(), handler);
        self
    }

    /// Démarrer tous les rafraîchissements
    pub fn start_all(self) {
        log("🚀 Starting refresh scheduler");
        
        for (id, handler) in self.handlers {
            let config = handler.config.clone();
            log(&format!("⏰ Scheduling refresh '{}' every {} seconds", id, config.interval_seconds));
            
            spawn_local(async move {
                // Exécuter immédiatement
                handler.execute_refresh().await;
                
                // Puis planifier les exécutions suivantes
                Self::schedule_refresh(handler).await;
            });
        }
    }

    /// Planifier un rafraîchissement périodique
    async fn schedule_refresh(handler: RefreshHandler) {
        let interval_ms = handler.config.interval_seconds * 1000;
        
        loop {
            // Attendre l'intervalle spécifié
            Self::sleep(interval_ms).await;
            
            // Exécuter le rafraîchissement
            handler.execute_refresh().await;
        }
    }

    /// Fonction utilitaire pour attendre (sleep) en millisecondes
    async fn sleep(ms: u32) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let window = web_sys::window().unwrap();
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
                .unwrap();
        });
        
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }
}

impl Default for RefreshScheduler {
    fn default() -> Self {
        Self::new()
    }
}
