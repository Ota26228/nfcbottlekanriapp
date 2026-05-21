pub mod app;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use leptos::prelude::*;

// ─── データ型定義 ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Shop {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Bottle {
    pub id: i32,
    pub shop_id: i32,
    pub nfc_uid: String,
    pub guest_name: Option<String>,
    pub drink_name: Option<String>,
    pub remaining_percent: i32,
    pub kept_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BottleCheckResult {
    NotRegistered { nfc_uid: String },
    Registered(Bottle),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Customer {
    pub id: i32,
    pub uuid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

// マイボトル一覧用（複数店舗横断）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MyBottleItem {
    pub id: i32,
    pub shop_name: String,
    pub nfc_uid: String,
    pub drink_name: Option<String>,
    pub remaining_percent: i32,
    pub kept_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ─── SSR専用: 状態管理 ───────────────────────────────────────

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use sqlx::SqlitePool;
    use leptos::prelude::provide_context;
    use webauthn_rs::prelude::{Webauthn, PasskeyRegistration, PasskeyAuthentication};

    #[derive(Clone)]
    pub struct WebauthnState {
        pub webauthn: Arc<Webauthn>,
        pub pending_regs:               Arc<Mutex<HashMap<String, PasskeyRegistration>>>,
        pub pending_auths:              Arc<Mutex<HashMap<String, PasskeyAuthentication>>>,
        pub pending_discoverable_auths: Arc<Mutex<HashMap<String, PasskeyAuthentication>>>,
    }

    pub fn register_db_pool(pool: SqlitePool) {
        provide_context(pool);
    }

    pub fn register_webauthn(state: WebauthnState) {
        provide_context(state);
    }
}

// ─── SSR専用: ヘルパー ───────────────────────────────────────

#[cfg(feature = "ssr")]
async fn extract_session_token() -> Option<String> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.ok()?;
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("session="))
        .map(|s| s["session=".len()..].to_string())
}

#[cfg(feature = "ssr")]
async fn get_authenticated_customer(
    pool: &sqlx::SqlitePool,
) -> Result<Customer, ServerFnError> {
    let token = extract_session_token().await
        .ok_or_else(|| ServerFnError::new("ログインが必要です"))?;

    sqlx::query_as::<_, Customer>(
        "SELECT c.id, c.uuid, c.email, c.display_name
         FROM customers c
         JOIN customer_sessions s ON s.customer_id = c.id
         WHERE s.token = ? AND s.expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("セッションが無効です"))
}

#[cfg(feature = "ssr")]
async fn send_magic_link_email(to_email: &str, link: &str) -> Result<(), String> {
    use lettre::{
        transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };

    let body = format!(
        "以下のリンクをタップしてログインしてください（15分以内）:\n\n{}\n\n心当たりがない場合は無視してください。",
        link
    );

    // SMTP_HOST が未設定の場合はコンソールに出力（開発・ハッカソン用）
    let smtp_host = match std::env::var("SMTP_HOST") {
        Ok(h) => h,
        Err(_) => {
            leptos::logging::log!("=== Magic Link (dev mode) ===");
            leptos::logging::log!("To: {}", to_email);
            leptos::logging::log!("Link: {}", link);
            leptos::logging::log!("==============================");
            return Ok(());
        }
    };

    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .ok().and_then(|p| p.parse().ok()).unwrap_or(587);
    let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();
    let from_addr = std::env::var("SMTP_FROM")
        .unwrap_or_else(|_| "noreply@bottlekanri.app".to_string());

    let email = Message::builder()
        .from(from_addr.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .to(to_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject("【ボトルキープ】ログインリンク")
        .body(body)
        .map_err(|e| e.to_string())?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
        .map_err(|e| e.to_string())?
        .port(smtp_port)
        .credentials(Credentials::new(smtp_user, smtp_pass))
        .build();

    mailer.send(email).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ─── WASMエントリポイント ─────────────────────────────────────

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

// ─── クライアント専用: NFC読み取り ───────────────────────────

#[cfg(feature = "hydrate")]
pub async fn nfc_scan() -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()
        .ok_or_else(|| "windowが見つかりません".to_string())?;

    let func = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("scanNfcTag"))
        .map_err(|_| "nfc_bridge.jsが読み込まれていません".to_string())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "scanNfcTagが関数ではありません".to_string())?;

    let promise = func
        .call0(&wasm_bindgen::JsValue::UNDEFINED)
        .map_err(|e| e.as_string().unwrap_or("呼び出しエラー".into()))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "Promiseが返されませんでした".to_string())?;

    JsFuture::from(promise)
        .await
        .map(|v| v.as_string().unwrap_or_default())
        .map_err(|e| e.as_string().unwrap_or("NFCエラー".into()))
}

// ─── クライアント専用: パスキーJS呼び出し ────────────────────

#[cfg(feature = "hydrate")]
pub async fn passkey_call_js(fn_name: &str, arg: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()
        .ok_or_else(|| "windowが見つかりません".to_string())?;

    let func = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str(fn_name))
        .map_err(|_| format!("passkey_bridge.jsが読み込まれていません ({fn_name})"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("{fn_name} が関数ではありません"))?;

    let arg_val = wasm_bindgen::JsValue::from_str(arg);
    let promise = func
        .call1(&wasm_bindgen::JsValue::UNDEFINED, &arg_val)
        .map_err(|e| e.as_string().unwrap_or("呼び出しエラー".into()))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "Promiseが返されませんでした".to_string())?;

    JsFuture::from(promise)
        .await
        .map(|v| v.as_string().unwrap_or_default())
        .map_err(|e| e.as_string().unwrap_or("パスキーエラー".into()))
}

// ─── サーバー関数: 既存 ───────────────────────────────────────

#[server(CheckNfcTag, "/api")]
pub async fn check_nfc_tag(nfc_uid: String) -> Result<BottleCheckResult, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DBプールが見つかりません"))?;

    let result = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE nfc_uid = ?"
    )
    .bind(&nfc_uid)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    match result {
        Some(bottle) if bottle.guest_name.is_some() => Ok(BottleCheckResult::Registered(bottle)),
        _ => Ok(BottleCheckResult::NotRegistered { nfc_uid }),
    }
}

#[server(RegisterBottle, "/api")]
pub async fn register_bottle(
    shop_id: i32,
    nfc_uid: String,
    guest_name: String,
    drink_name: String,
    expires_days: i32,
) -> Result<Bottle, ServerFnError> {
    use leptos::prelude::use_context;
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DBプールが見つかりません"))?;

    let now = Utc::now();
    let expires_at = now + Duration::days(expires_days as i64);

    let result = sqlx::query(
        "INSERT INTO bottles (shop_id, nfc_uid, guest_name, drink_name,
                              kept_at, expires_at, remaining_percent)
         VALUES (?, ?, ?, ?, ?, ?, 100)"
    )
    .bind(shop_id)
    .bind(&nfc_uid)
    .bind(&guest_name)
    .bind(&drink_name)
    .bind(now)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB登録エラー: {}", e)))?;

    let bottle = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(bottle)
}

#[server(GetShopBottles, "/api")]
pub async fn get_shop_bottles(shop_id: i32) -> Result<Vec<Bottle>, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DBプールが見つかりません"))?;

    let bottles = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE shop_id = ?
         ORDER BY kept_at DESC"
    )
    .bind(shop_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(bottles)
}

#[server(GetShop, "/api")]
pub async fn get_shop(shop_id: i32) -> Result<Shop, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DBプールが見つかりません"))?;

    let shop = sqlx::query_as::<_, Shop>("SELECT id, name FROM shops WHERE id = ?")
        .bind(shop_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("店舗が見つかりません"))?;

    Ok(shop)
}

// ─── サーバー関数: メール認証 ─────────────────────────────────

#[server(RequestMagicLink, "/api")]
pub async fn request_magic_link(email: String) -> Result<(), ServerFnError> {
    use leptos::prelude::use_context;
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    // メールアドレスを小文字正規化
    let email = email.trim().to_lowercase();

    // 古いリンクを削除
    sqlx::query("DELETE FROM auth_magic_links WHERE email = ? OR expires_at < datetime('now')")
        .bind(&email)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(15);

    sqlx::query(
        "INSERT INTO auth_magic_links (token, email, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&token)
    .bind(&email)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let link = format!("{}/auth/verify?token={}", app_url, token);

    send_magic_link_email(&email, &link)
        .await
        .map_err(ServerFnError::new)?;

    Ok(())
}

#[server(VerifyMagicLink, "/api")]
pub async fn verify_magic_link(token: String) -> Result<Customer, ServerFnError> {
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    // トークン検証
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT token, email FROM auth_magic_links
         WHERE token = ? AND used = 0 AND expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("リンクが無効または期限切れです"))?;

    let email = row.1;

    // 使用済みにする
    sqlx::query("UPDATE auth_magic_links SET used = 1 WHERE token = ?")
        .bind(&token)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // 顧客を取得または作成
    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let customer = if let Some(c) = customer {
        c
    } else {
        let new_uuid = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO customers (uuid, email) VALUES (?, ?)")
            .bind(&new_uuid)
            .bind(&email)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        sqlx::query_as::<_, Customer>(
            "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
        )
        .bind(&email)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    };

    // セッション作成
    let session_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query(
        "INSERT INTO customer_sessions (token, customer_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&session_token)
    .bind(customer.id)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // httpOnly Cookie をセット
    let response_opts = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not found"))?;
    let cookie = format!(
        "session={session_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000"
    );
    response_opts.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(customer)
}

#[server(GetCurrentCustomer, "/api")]
pub async fn get_current_customer() -> Result<Option<Customer>, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    let token = match extract_session_token().await {
        Some(t) => t,
        None => return Ok(None),
    };

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT c.id, c.uuid, c.email, c.display_name
         FROM customers c
         JOIN customer_sessions s ON s.customer_id = c.id
         WHERE s.token = ? AND s.expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(customer)
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    if let Some(token) = extract_session_token().await {
        sqlx::query("DELETE FROM customer_sessions WHERE token = ?")
            .bind(&token)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    let response_opts = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not found"))?;
    response_opts.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_static(
            "session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"
        ),
    );

    Ok(())
}

// ─── サーバー関数: 顧客ボトル ─────────────────────────────────

#[server(GetMyBottles, "/api")]
pub async fn get_my_bottles() -> Result<Vec<MyBottleItem>, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    let customer = get_authenticated_customer(&pool).await?;

    let items = sqlx::query_as::<_, MyBottleItem>(
        "SELECT b.id, s.name as shop_name, b.nfc_uid, b.drink_name,
                b.remaining_percent, b.kept_at, b.expires_at
         FROM customer_bottles cb
         JOIN bottles b ON b.id = cb.bottle_id
         JOIN shops s ON s.id = b.shop_id
         WHERE cb.customer_id = ?
         ORDER BY s.name, b.kept_at DESC"
    )
    .bind(customer.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(items)
}

#[server(LinkBottle, "/api")]
pub async fn link_bottle(nfc_uid: String) -> Result<MyBottleItem, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    let customer = get_authenticated_customer(&pool).await?;

    let bottle = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE nfc_uid = ?"
    )
    .bind(&nfc_uid)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("このタグは未登録です。スタッフにお声がけください。"))?;

    // 紐づけ（既に紐づいていても無視）
    sqlx::query(
        "INSERT OR IGNORE INTO customer_bottles (customer_id, bottle_id) VALUES (?, ?)"
    )
    .bind(customer.id)
    .bind(bottle.id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let item = sqlx::query_as::<_, MyBottleItem>(
        "SELECT b.id, s.name as shop_name, b.nfc_uid, b.drink_name,
                b.remaining_percent, b.kept_at, b.expires_at
         FROM bottles b
         JOIN shops s ON s.id = b.shop_id
         WHERE b.id = ?"
    )
    .bind(bottle.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(item)
}

// ─── サーバー関数: パスキー ───────────────────────────────────

#[server(HasPasskey, "/api")]
pub async fn has_passkey() -> Result<bool, ServerFnError> {
    use leptos::prelude::use_context;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;

    let customer = get_authenticated_customer(&pool).await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM passkey_credentials WHERE customer_id = ?"
    )
    .bind(customer.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(count.0 > 0)
}

#[server(PasskeyRegisterStart, "/api")]
pub async fn passkey_register_start() -> Result<String, ServerFnError> {
    use leptos::prelude::use_context;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::Uuid;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let customer = get_authenticated_customer(&pool).await?;
    let user_uuid = Uuid::parse_str(&customer.uuid)
        .map_err(|_| ServerFnError::new("UUID parse error"))?;

    let username = customer.email.as_deref().unwrap_or(&customer.uuid).to_string();
    let display = customer.display_name.as_deref().unwrap_or(&username).to_string();

    let (ccr, reg_state) = state.webauthn
        .start_passkey_registration(user_uuid, &username, &display, None)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    state.pending_regs.lock().unwrap()
        .insert(customer.uuid.clone(), reg_state);

    serde_json::to_string(&ccr)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(PasskeyRegisterFinish, "/api")]
pub async fn passkey_register_finish(response_json: String) -> Result<(), ServerFnError> {
    use leptos::prelude::use_context;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::RegisterPublicKeyCredential;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let customer = get_authenticated_customer(&pool).await?;

    let reg_state = state.pending_regs.lock().unwrap()
        .remove(&customer.uuid)
        .ok_or_else(|| ServerFnError::new("登録セッションが見つかりません"))?;

    let reg_response: RegisterPublicKeyCredential = serde_json::from_str(&response_json)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let passkey = state.webauthn
        .finish_passkey_registration(&reg_response, &reg_state)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let public_key = serde_json::to_string(&passkey)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "INSERT INTO passkey_credentials (customer_id, public_key) VALUES (?, ?)"
    )
    .bind(customer.id)
    .bind(&public_key)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server(PasskeyLoginStart, "/api")]
pub async fn passkey_login_start(email: String) -> Result<String, ServerFnError> {
    use leptos::prelude::use_context;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::Passkey;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let email = email.trim().to_lowercase();

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("メールアドレスが見つかりません"))?;

    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT public_key FROM passkey_credentials WHERE customer_id = ?"
    )
    .bind(customer.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if rows.is_empty() {
        return Err(ServerFnError::new("パスキーが登録されていません"));
    }

    let passkeys: Vec<Passkey> = rows.iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    let (rcr, auth_state) = state.webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    state.pending_auths.lock().unwrap()
        .insert(email.clone(), auth_state);

    serde_json::to_string(&rcr)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(PasskeyLoginFinish, "/api")]
pub async fn passkey_login_finish(
    email: String,
    response_json: String,
) -> Result<Customer, ServerFnError> {
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::PublicKeyCredential;
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let email = email.trim().to_lowercase();

    let auth_state = state.pending_auths.lock().unwrap()
        .remove(&email)
        .ok_or_else(|| ServerFnError::new("認証セッションが見つかりません"))?;

    let auth_response: PublicKeyCredential = serde_json::from_str(&response_json)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    state.webauthn
        .finish_passkey_authentication(&auth_response, &auth_state)
        .map_err(|_| ServerFnError::new("パスキー認証に失敗しました"))?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
    )
    .bind(&email)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // セッション作成
    let session_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query(
        "INSERT INTO customer_sessions (token, customer_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&session_token)
    .bind(customer.id)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let response_opts = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not found"))?;
    let cookie = format!(
        "session={session_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000"
    );
    response_opts.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(customer)
}

// ─── サーバー関数: 新規ユーザーパスキー登録 ─────────────────────

#[server(PasskeyFirstRegStart, "/api")]
pub async fn passkey_first_reg_start(
    email: Option<String>,
    display_name: Option<String>,
) -> Result<(String, String), ServerFnError> {
    use leptos::prelude::use_context;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::Uuid as WebauthnUuid;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let email_normalized = email.as_deref().map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty());

    // メールアドレスがある場合は既存アカウント検索
    let existing = if let Some(ref e) = email_normalized {
        sqlx::query_as::<_, Customer>(
            "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
        )
        .bind(e)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    } else {
        None
    };

    let customer = match existing {
        Some(c) => c,
        None => {
            let new_uuid = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO customers (uuid, email, display_name) VALUES (?, ?, ?)"
            )
            .bind(&new_uuid)
            .bind(&email_normalized)
            .bind(&display_name)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            sqlx::query_as::<_, Customer>(
                "SELECT id, uuid, email, display_name FROM customers WHERE uuid = ?"
            )
            .bind(&new_uuid)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        }
    };

    let user_uuid = WebauthnUuid::parse_str(&customer.uuid)
        .map_err(|_| ServerFnError::new("UUID parse error"))?;

    // WebAuthn username にはメールか UUID を使用
    let username = customer.email.as_deref().unwrap_or(&customer.uuid).to_string();
    let display = customer.display_name.as_deref().unwrap_or(&username).to_string();

    let (ccr, reg_state) = state.webauthn
        .start_passkey_registration(user_uuid, &username, &display, None)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    state.pending_regs.lock().unwrap()
        .insert(customer.uuid.clone(), reg_state);

    let challenge_json = serde_json::to_string(&ccr)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((customer.uuid, challenge_json))
}

#[server(PasskeyFirstRegFinish, "/api")]
pub async fn passkey_first_reg_finish(
    customer_uuid: String,
    response_json: String,
) -> Result<Customer, ServerFnError> {
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::RegisterPublicKeyCredential;
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let reg_state = state.pending_regs.lock().unwrap()
        .remove(&customer_uuid)
        .ok_or_else(|| ServerFnError::new("登録セッションが見つかりません"))?;

    let reg_response: RegisterPublicKeyCredential = serde_json::from_str(&response_json)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let passkey = state.webauthn
        .finish_passkey_registration(&reg_response, &reg_state)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let public_key = serde_json::to_string(&passkey)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE uuid = ?"
    )
    .bind(&customer_uuid)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "INSERT INTO passkey_credentials (customer_id, public_key) VALUES (?, ?)"
    )
    .bind(customer.id)
    .bind(&public_key)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let session_token2 = uuid::Uuid::new_v4().to_string();
    let expires_at2 = Utc::now() + Duration::days(30);

    sqlx::query(
        "INSERT INTO customer_sessions (token, customer_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&session_token2)
    .bind(customer.id)
    .bind(expires_at2)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let resp_opts = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not found"))?;
    let cookie2 = format!(
        "session={session_token2}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000"
    );
    resp_opts.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie2)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(customer)
}

// ─── サーバー関数: メールなしパスキーログイン（discoverable）────
// webauthn-rs 0.5 には start_discoverable_authentication がないため、
// DB 上の全パスキーを allowCredentials に乗せて同等の動作を実現する。

#[server(PasskeyDiscoverableStart, "/api")]
pub async fn passkey_discoverable_start() -> Result<(String, String), ServerFnError> {
    use leptos::prelude::use_context;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::Passkey;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    // DB上の全パスキーを取得
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT public_key FROM passkey_credentials"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let passkeys: Vec<Passkey> = rows.iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    if passkeys.is_empty() {
        return Err(ServerFnError::new("登録済みのパスキーがありません"));
    }

    let (rcr, auth_state) = state.webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    state.pending_discoverable_auths.lock().unwrap()
        .insert(session_id.clone(), auth_state);

    let challenge_json = serde_json::to_string(&rcr)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((session_id, challenge_json))
}

#[server(PasskeyDiscoverableFinish, "/api")]
pub async fn passkey_discoverable_finish(
    session_id: String,
    response_json: String,
) -> Result<Customer, ServerFnError> {
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use crate::ssr::WebauthnState;
    use webauthn_rs::prelude::{PublicKeyCredential, Passkey};
    use chrono::Duration;

    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("DB not found"))?;
    let state = use_context::<WebauthnState>()
        .ok_or_else(|| ServerFnError::new("WebAuthn not initialized"))?;

    let auth_state = state.pending_discoverable_auths.lock().unwrap()
        .remove(&session_id)
        .ok_or_else(|| ServerFnError::new("認証セッションが見つかりません"))?;

    let auth_response: PublicKeyCredential = serde_json::from_str(&response_json)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let auth_result = state.webauthn
        .finish_passkey_authentication(&auth_response, &auth_state)
        .map_err(|_| ServerFnError::new("パスキー認証に失敗しました"))?;

    // 使われたクレデンシャルIDから顧客を特定
    let used_cred_id = auth_result.cred_id();
    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT customer_id, public_key FROM passkey_credentials"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let customer_id = rows.iter()
        .find_map(|(cid, json)| {
            serde_json::from_str::<Passkey>(json).ok()
                .filter(|pk| pk.cred_id() == used_cred_id)
                .map(|_| *cid)
        })
        .ok_or_else(|| ServerFnError::new("顧客が見つかりません"))?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE id = ?"
    )
    .bind(customer_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let session_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query(
        "INSERT INTO customer_sessions (token, customer_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&session_token)
    .bind(customer.id)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let resp_opts = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not found"))?;
    let cookie = format!(
        "session={session_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000"
    );
    resp_opts.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie)
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(customer)
}
