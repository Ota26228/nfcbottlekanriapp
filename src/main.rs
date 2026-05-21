#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use webauthn_rs::prelude::WebauthnBuilder;
    use url::Url;
    use bottlekanri::app::*;
    use bottlekanri::ssr::{WebauthnState, register_db_pool, register_webauthn};

    // ─── DBセットアップ ───────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./bottles.db?mode=rwc".to_string());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("DBへの接続に失敗しました");

    sqlx::query(include_str!("../schema.sql"))
        .execute(&pool)
        .await
        .expect("スキーマの作成に失敗しました");

    sqlx::query("INSERT OR IGNORE INTO shops (id, name) VALUES (1, 'デモバー')")
        .execute(&pool)
        .await
        .expect("デモデータの作成に失敗しました");

    log!("DB初期化完了");

    // ─── WebAuthn（パスキー）セットアップ ────────────────────
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let rp_id = std::env::var("RP_ID")
        .unwrap_or_else(|_| {
            Url::parse(&app_url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_else(|| "localhost".to_string())
        });

    let rp_origin = Url::parse(&app_url).expect("APP_URL が不正です");

    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)
        .expect("WebAuthn 設定エラー")
        .rp_name("ボトルキープ")
        .build()
        .expect("WebAuthn ビルド失敗");

    let webauthn_state = WebauthnState {
        webauthn: Arc::new(webauthn),
        pending_regs:               Arc::new(Mutex::new(HashMap::new())),
        pending_auths:              Arc::new(Mutex::new(HashMap::new())),
        pending_discoverable_auths: Arc::new(Mutex::new(HashMap::new())),
    };

    log!("WebAuthn初期化完了 (rp_id={})", rp_id);

    // ─── Leptosルーター設定 ───────────────────────────────────
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                let webauthn_state = webauthn_state.clone();
                move || {
                    register_db_pool(pool.clone());
                    register_webauthn(webauthn_state.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("サーバー起動: http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
