use actix_web::{web, HttpRequest, HttpResponse, Result, http::header};
use awc::Client;
use regex::Regex;
use std::sync::{OnceLock, atomic::{AtomicU64, Ordering}};
use log;

// Compteurs statistiques globaux
static BLOCKED_REQUESTS_COUNT: AtomicU64 = AtomicU64::new(0);
static PROCESSED_REQUESTS_COUNT: AtomicU64 = AtomicU64::new(0);
static HTML_CLEANINGS_COUNT: AtomicU64 = AtomicU64::new(0);

// Domaines publicitaires YouTube à bloquer
static BLOCKED_DOMAINS: &[&str] = &[
    "googleads.g.doubleclick.net",
    "googlesyndication.com",
    "youtube.com/pagead",
    "youtube.com/get_video_info",
    "s.youtube.com/api/stats",
    "youtube.com/api/stats",
    "youtube.com/ptracking",
    "youtube.com/youtubei/v1/log_event",
    "youtube.com/generate_204",
    "yt3.ggpht.com/ytts/",
    "i.ytimg.com/an_webp/",
];

// CSS pour masquer les publicités
static AD_BLOCK_CSS: &str = r#"<style>
/* Masquer les publicités YouTube */
#player-ads, .ytp-ad-module, .video-ads, .ytp-ad-overlay-container,
.ytp-ad-text-overlay, .ytp-ad-skip-button, .ytp-ad-overlay-close-button,
.ytp-ad-image-overlay, .masthead-ad-control, .ytd-promoted-sparkles-web-renderer,
.ytd-promoted-video-renderer, .ytd-compact-promoted-video-renderer,
.ytd-ad-slot-renderer, .ytd-display-ad-renderer, .ytd-in-feed-ad-layout-renderer,
.ytd-banner-promo-renderer, .ytd-statement-banner-renderer,
#masthead-ad, #footer-ads, #watch-sidebar-ads, .promoted-sparkles,
.ytd-promoted-sparkles-text-search-renderer, .ytd-search-pyv-renderer,
ytd-action-companion-ad-renderer, ytd-display-ad-renderer,
ytd-video-masthead-ad-advertiser-info-renderer, ytd-video-masthead-ad-primary-video-renderer,
.ytd-merch-shelf-renderer, .ytd-player-legacy-desktop-watch-ads-renderer {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    height: 0 !important;
    width: 0 !important;
}

/* Masquer les overlays publicitaires */
.ytp-ad-overlay-container *, .ytp-ad-text-overlay *, 
.ytp-ad-image-overlay *, .ytp-pause-overlay * {
    display: none !important;
}

/* Accélérer les publicités qui ne peuvent pas être complètement bloquées */
.video-stream.html5-main-video {
    filter: contrast(0) !important;
}

/* Masquer les suggestions sponsorisées */
[class*="promoted"], [class*="sponsor"], [data-is-sponsor="true"], 
[aria-label*="Sponsored"], [aria-label*="Ad"] {
    display: none !important;
}
</style>"#;

// Regex pour nettoyer le HTML
static HTML_CLEANER: OnceLock<Regex> = OnceLock::new();

fn get_html_cleaner() -> &'static Regex {
    HTML_CLEANER.get_or_init(|| {
        Regex::new(r#"(?i)<script[^>]*(?:google|doubleclick|ads|analytics|adservice)[^>]*>.*?</script>|<iframe[^>]*(?:google|doubleclick|ads)[^>]*>.*?</iframe>|<div[^>]*(?:ad-|ads-|advertisement)[^>]*>.*?</div>"#).unwrap()
    })
}

pub fn is_blocked_url(url: &str) -> bool {
    for domain in BLOCKED_DOMAINS.iter() {
        if url.contains(domain) {
            BLOCKED_REQUESTS_COUNT.fetch_add(1, Ordering::Relaxed);
            log::warn!("🚫 BLOCKED: {} (matched domain: {}) [Total blocked: {}]", 
                url, domain, BLOCKED_REQUESTS_COUNT.load(Ordering::Relaxed));
            return true;
        }
    }
    false
}

pub async fn proxy_youtube(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    let target_path = path.into_inner();
    
    // Déterminer l'URL cible
    let target_url = if target_path.starts_with("http://") || target_path.starts_with("https://") {
        // L'URL est déjà complète
        target_path
    } else {
        // Ajouter le préfixe YouTube
        format!("https://www.youtube.com/{}", target_path)
    };
    
    // Incrémenter le compteur de requêtes traitées
    PROCESSED_REQUESTS_COUNT.fetch_add(1, Ordering::Relaxed);
    
    // Bloquer les URLs publicitaires
    if is_blocked_url(&target_url) {
        log::warn!("🚫 AD BLOCKED: {}", target_url);
        return Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body("<!-- Ad blocked by Rust Proxy -->"));
    }
    
    // Construire la requête vers YouTube
    let query_string = req.query_string();
    let full_url = if query_string.is_empty() {
        target_url.clone()
    } else {
        format!("{}?{}", target_url, query_string)
    };
    
    log::info!("✅ PROXYING: {} -> {}", req.method(), full_url);
    
    // Créer la requête
    let mut client_req = match req.method().as_str() {
        "GET" => client.get(&full_url),
        "POST" => client.post(&full_url),
        "PUT" => client.put(&full_url),
        "DELETE" => client.delete(&full_url),
        "HEAD" => client.head(&full_url),
        _ => client.get(&full_url),
    };
    
    // Copier les headers (en excluant ceux qui peuvent causer des problèmes)
    for (name, value) in req.headers().iter() {
        let header_name = name.as_str().to_lowercase();
        if !matches!(header_name.as_str(), 
            "host" | "content-length" | "transfer-encoding" | "connection" | "upgrade") {
            client_req = client_req.insert_header((name.clone(), value.clone()));
        }
    }
    
    // Ajouter des headers pour se faire passer pour un navigateur normal
    client_req = client_req
        .insert_header(("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"))
        .insert_header(("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"))
        .insert_header(("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8"))
        .insert_header(("Accept-Encoding", "gzip, deflate, br"))
        .insert_header(("DNT", "1"))
        .insert_header(("Upgrade-Insecure-Requests", "1"))
        .insert_header(("Sec-Fetch-Dest", "document"))
        .insert_header(("Sec-Fetch-Mode", "navigate"))
        .insert_header(("Sec-Fetch-Site", "none"));
      // Envoyer la requête
    let mut response = match client_req.send_body(body).await {
        Ok(res) => {
            log::info!("📡 Response received: {} (status: {})", full_url, res.status());
            res
        },
        Err(e) => {
            log::error!("❌ Failed to proxy request to {}: {}", full_url, e);
            return Ok(HttpResponse::BadGateway()
                .json(serde_json::json!({"error": "Failed to proxy request"})));
        }
    };
    
    // Récupérer le contenu de la réponse
    let response_body = match response.body().await {
        Ok(body) => {
            log::debug!("📦 Response body size: {} bytes", body.len());
            body
        },
        Err(e) => {
            log::error!("❌ Failed to read response body from {}: {}", full_url, e);
            return Ok(HttpResponse::BadGateway()
                .json(serde_json::json!({"error": "Failed to read response"})));
        }
    };
    
    // Construire la réponse
    let mut http_response = HttpResponse::build(response.status());
    
    // Copier les headers de réponse (en excluant ceux problématiques)
    for (name, value) in response.headers().iter() {
        let header_name = name.as_str().to_lowercase();
        if !matches!(header_name.as_str(), 
            "content-length" | "transfer-encoding" | "content-encoding" | 
            "connection" | "upgrade" | "content-security-policy") {
            http_response.insert_header((name.clone(), value.clone()));
        }
    }
      // Traitement spécial pour le HTML
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        if content_type.to_str().unwrap_or("").contains("text/html") {
            log::info!("🔧 Processing HTML content for ad blocking");
            let html_content = String::from_utf8_lossy(&response_body);
            
            // Nettoyer le HTML des scripts publicitaires
            let original_size = html_content.len();
            let cleaned_html = get_html_cleaner().replace_all(&html_content, "");
            let cleaned_size = cleaned_html.len();
              if original_size != cleaned_size {
                HTML_CLEANINGS_COUNT.fetch_add(1, Ordering::Relaxed);
                log::warn!("🧹 CLEANED HTML: Removed {} bytes of ad content [Total cleanings: {}]", 
                    original_size - cleaned_size, HTML_CLEANINGS_COUNT.load(Ordering::Relaxed));
            }
              // Injecter le CSS anti-publicité
            let final_html = if cleaned_html.contains("</head>") {
                log::info!("💉 INJECTING: AdBlock CSS into HTML <head>");
                cleaned_html.replace("</head>", &format!("{}\n</head>", AD_BLOCK_CSS))
            } else {
                log::info!("💉 INJECTING: AdBlock CSS at top of HTML (no <head> found)");
                format!("{}\n{}", AD_BLOCK_CSS, cleaned_html)
            };
            
            log::info!("✅ HTML PROCESSED: Final size {} bytes", final_html.len());
            return Ok(http_response.body(final_html));
        }
    }
      // Pour les autres types de contenu, retourner tel quel
    log::debug!("📄 Non-HTML content, passing through without modification");
    Ok(http_response.body(response_body))
}

pub async fn adblock_status() -> Result<HttpResponse> {
    log::info!("📊 AdBlock status requested");
    
    let blocked_count = BLOCKED_REQUESTS_COUNT.load(Ordering::Relaxed);
    let processed_count = PROCESSED_REQUESTS_COUNT.load(Ordering::Relaxed);
    let cleanings_count = HTML_CLEANINGS_COUNT.load(Ordering::Relaxed);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "active",
        "message": "YouTube AdBlock Proxy is running",
        "statistics": {
            "total_requests_processed": processed_count,
            "ads_blocked": blocked_count,
            "html_cleanings_performed": cleanings_count,
            "block_rate_percentage": if processed_count > 0 {
                (blocked_count as f64 / processed_count as f64 * 100.0).round()
            } else { 0.0 }
        },
        "blocked_domains": BLOCKED_DOMAINS.len(),
        "blocked_domains_list": BLOCKED_DOMAINS,
        "features": [
            "CSS injection anti-ads",
            "HTML cleaning (scripts removal)",
            "Domain blocking",
            "Request filtering"
        ],
        "proxy_endpoint": "/proxy/youtube/{path}",
        "test_url": "http://127.0.0.1:8089/proxy/youtube/",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

pub fn configure_proxy_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/proxy")
            .route("/youtube/{path:.*}", web::to(proxy_youtube))
            .route("/adblock/status", web::get().to(adblock_status))
    );
}
