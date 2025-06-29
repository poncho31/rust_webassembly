// Re-export des fonctions et structures du serveur web principal
// Ce module permet de partager le code du serveur avec d'autres composants

pub use crate::main::{start_full_web_server, WebServerConfig, create_web_server_config};
