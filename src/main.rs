use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::{routing::{get, patch, post}, Router};
use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};
use url::Url;
use webauthn_rs::prelude::WebauthnBuilder;
use bcrypt;

use bottlekanri::{
    AppState,
    handler_list_shops, handler_get_shop, handler_register_shop,
    handler_request_magic_link, handler_verify_magic_link,
    handler_get_me, handler_logout, handler_update_profile, handler_passkey_status,
    handler_passkey_reg_start, handler_passkey_reg_finish,
    handler_passkey_login_start, handler_passkey_login_finish,
    handler_staff_login, handler_staff_me, handler_staff_logout,
    handler_register_bottle, handler_get_shop_bottles, handler_update_bottle,
    handler_get_my_bottles,
    handler_analyze_bottle_image,
    handler_notify_test,
    handler_admin_clear_magic_links,
    run_notification_loop,
};



#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // ─── DB ──────────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./bottles.db?mode=rwc".to_string());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("DBへの接続に失敗しました");

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool).await.expect("WALモード設定失敗");

    sqlx::query(include_str!("../schema.sql"))
        .execute(&pool).await.expect("スキーマの作成に失敗しました");

    // nfc_uid カラム削除マイグレーション
    let has_nfc: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('bottles') WHERE name='nfc_uid'"
    ).fetch_one(&pool).await.unwrap_or(0);
    if has_nfc > 0 {
        sqlx::query("CREATE TABLE IF NOT EXISTS bottles_v2 (
            id INTEGER PRIMARY KEY AUTOINCREMENT, shop_id INTEGER NOT NULL,
            guest_name TEXT, drink_name TEXT, remaining_percent INTEGER DEFAULT 100,
            kept_at DATETIME, expires_at DATETIME, email TEXT,
            FOREIGN KEY (shop_id) REFERENCES shops(id)
        )").execute(&pool).await.expect("bottles_v2 作成失敗");
        sqlx::query("INSERT INTO bottles_v2 (id,shop_id,guest_name,drink_name,remaining_percent,kept_at,expires_at,email)
            SELECT id,shop_id,guest_name,drink_name,remaining_percent,kept_at,expires_at,email FROM bottles")
            .execute(&pool).await.expect("データ移行失敗");
        sqlx::query("DROP TABLE bottles").execute(&pool).await.expect("旧テーブル削除失敗");
        sqlx::query("ALTER TABLE bottles_v2 RENAME TO bottles").execute(&pool).await.expect("リネーム失敗");
        tracing::info!("nfc_uid マイグレーション完了");
    }

    let demo_pin = std::env::var("DEMO_PIN").unwrap_or_else(|_| "1234".to_string());
    let hashed_pin = bcrypt::hash(&demo_pin, bcrypt::DEFAULT_COST)
        .expect("ハッシュ化失敗");

    sqlx::query("INSERT OR IGNORE INTO shops (id, name, pin) VALUES (1, 'デモバー',?)")
        .bind(&hashed_pin)
        .execute(&pool).
        await
        .expect("デモデータの作成に失敗しました");


    tracing::info!("DB初期化完了");

    // ─── WebAuthn ────────────────────────────────────────────
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| {
        Url::parse(&app_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| "localhost".to_string())
    });

    let rp_origin = Url::parse(&app_url).expect("APP_URL が不正です");

    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)
        .expect("WebAuthn設定エラー")
        .rp_name("ボトルキープ")
        .build()
        .expect("WebAuthnビルド失敗");

    tracing::info!("WebAuthn初期化完了 (rp_id={})", rp_id);

    let state = AppState {
        pool,
        webauthn:                   Arc::new(webauthn),
        pending_regs:               Arc::new(Mutex::new(HashMap::new())),
        pending_auths:              Arc::new(Mutex::new(HashMap::new())),
        pending_discoverable_auths: Arc::new(Mutex::new(HashMap::new())),
    };

    // ─── CORS ────────────────────────────────────────────────
    let cors = if let Ok(origin) = std::env::var("CORS_ORIGIN") {
        let value = origin.parse::<HeaderValue>().expect("CORS_ORIGIN が不正なURL形式です");
        CorsLayer::new()
            .allow_origin(value)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // ─── ルーター ────────────────────────────────────────────
    let app = Router::new()
        // ショップ
        .route("/v1/shops", get(handler_list_shops).post(handler_register_shop))
        .route("/v1/shops/{shop_id}", get(handler_get_shop))
        // 顧客認証
        .route("/v1/auth/magic-link/send",    post(handler_request_magic_link))
        .route("/v1/auth/magic-link/verify",  post(handler_verify_magic_link))
        .route("/v1/auth/me",                 get(handler_get_me))
        .route("/v1/auth/passkey/status",     get(handler_passkey_status))
        .route("/v1/auth/logout",             post(handler_logout))
        .route("/v1/auth/profile",            patch(handler_update_profile))
        // パスキー
        .route("/v1/auth/passkey/register/start",  post(handler_passkey_reg_start))
        .route("/v1/auth/passkey/register/finish", post(handler_passkey_reg_finish))
        .route("/v1/auth/passkey/login/start",     post(handler_passkey_login_start))
        .route("/v1/auth/passkey/login/finish",    post(handler_passkey_login_finish))
        // スタッフ
        .route("/v1/staff/login",   post(handler_staff_login))
        .route("/v1/staff/me",      get(handler_staff_me))
        .route("/v1/staff/logout",  post(handler_staff_logout))
        .route("/v1/staff/bottles",
            get(handler_get_shop_bottles).post(handler_register_bottle)
        )
        .route("/v1/staff/bottles/{id}", patch(handler_update_bottle))
        // AI画像解析
        .route("/v1/staff/bottles/analyze-image", post(handler_analyze_bottle_image))
        // テスト用（通知即時実行）
        .route("/v1/staff/notify-test", post(handler_notify_test))
        // 一時管理用
        .route("/v1/admin/clear-magic-links", post(handler_admin_clear_magic_links))
        // 顧客ボトル
        .route("/v1/customer/bottles",      get(handler_get_my_bottles))
        .layer(cors)
        .with_state(state.clone());

    // ─── 起動 ────────────────────────────────────────────────
    let addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    tokio::spawn(run_notification_loop(state.pool.clone()));

    tracing::info!("サーバー起動: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
