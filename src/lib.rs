use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use webauthn_rs::prelude::{Webauthn, PasskeyRegistration, PasskeyAuthentication};

// ─── 共有状態 ──────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub webauthn: Arc<Webauthn>,
    pub pending_regs:               Arc<Mutex<HashMap<String, PasskeyRegistration>>>,
    pub pending_auths:              Arc<Mutex<HashMap<String, PasskeyAuthentication>>>,
    pub pending_discoverable_auths: Arc<Mutex<HashMap<String, PasskeyAuthentication>>>,
}

// ─── エラー型 ──────────────────────────────────────────────────

pub enum ApiError {
    Unauthorized(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Unauthorized(m) => write!(f, "{}", m),
            ApiError::NotFound(m)     => write!(f, "{}", m),
            ApiError::BadRequest(m)   => write!(f, "{}", m),
            ApiError::Internal(m)     => write!(f, "{}", m),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            ApiError::NotFound(m)     => (StatusCode::NOT_FOUND, m),
            ApiError::BadRequest(m)   => (StatusCode::BAD_REQUEST, m),
            ApiError::Internal(m)     => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

// ─── データ型 ──────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shop {
    pub id:   i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bottle {
    pub id:                i32,
    pub shop_id:           i32,
    pub nfc_uid:           String,
    pub guest_name:        Option<String>,
    pub drink_name:        Option<String>,
    pub remaining_percent: i32,
    pub kept_at:           Option<DateTime<Utc>>,
    pub expires_at:        Option<DateTime<Utc>>,
    pub email:             Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Customer {
    pub id:           i32,
    pub uuid:         String,
    pub email:        Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MyBottleItem {
    pub id:                i32,
    pub shop_name:         String,
    pub nfc_uid:           String,
    pub drink_name:        Option<String>,
    pub remaining_percent: i32,
    pub kept_at:           Option<DateTime<Utc>>,
    pub expires_at:        Option<DateTime<Utc>>,
}

// ─── 認証ヘルパー ──────────────────────────────────────────────

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

async fn get_authenticated_customer(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<Customer, ApiError> {
    let token = extract_bearer(headers)
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".into()))?;

    sqlx::query_as::<_, Customer>(
        "SELECT c.id, c.uuid, c.email, c.display_name
         FROM customers c
         JOIN customer_sessions s ON s.customer_id = c.id
         WHERE s.token = ? AND s.expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::Unauthorized("セッションが無効です".into()))
}

async fn get_authenticated_staff(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<i32, ApiError> {
    let token = extract_bearer(headers)
        .ok_or_else(|| ApiError::Unauthorized("スタッフ認証が必要です".into()))?;

    sqlx::query_scalar::<_, i32>(
        "SELECT shop_id FROM staff_sessions WHERE token = ? AND expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::Unauthorized("スタッフセッションが無効です".into()))
}

async fn create_customer_session(pool: &SqlitePool, customer_id: i32) -> Result<String, ApiError> {
    let token      = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);
    sqlx::query(
        "INSERT INTO customer_sessions (token, customer_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&token)
    .bind(customer_id)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;
    Ok(token)
}

// ─── メール送信 ────────────────────────────────────────────────

async fn send_magic_link_email(to_email: &str, link: &str) -> Result<(), ApiError> {
    let api_key = match std::env::var("RESEND_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            tracing::info!("=== Magic Link (dev mode) === To: {} | Link: {}", to_email, link);
            return Ok(());
        }
    };

    let from_addr = std::env::var("RESEND_FROM")
        .unwrap_or_else(|_| "onboarding@resend.dev".to_string());

    let body = format!(
        "以下のリンクをタップしてログインしてください（15分以内）:\n\n{}\n\n心当たりがない場合は無視してください。",
        link
    );

    let payload = serde_json::json!({
        "from": from_addr,
        "to": [to_email],
        "subject": "【ボトルキープ】ログインリンク",
        "text": body,
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.resend.com/emails")
        .bearer_auth(&api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !res.status().is_success() {
        let msg = res.text().await.unwrap_or_default();
        return Err(ApiError::Internal(format!("Resend error: {}", msg)));
    }

    Ok(())
}

// ════════════════════════════════════════════════════════════
// ハンドラー — ショップ
// ════════════════════════════════════════════════════════════

// GET /v1/shops/:shop_id
pub async fn handler_get_shop(
    State(state): State<AppState>,
    Path(shop_id): Path<i32>,
) -> ApiResult<Shop> {
    let shop = sqlx::query_as::<_, Shop>("SELECT id, name FROM shops WHERE id = ?")
        .bind(shop_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("店舗が見つかりません".into()))?;
    Ok(Json(shop))
}

// ════════════════════════════════════════════════════════════
// ハンドラー — 顧客認証
// ════════════════════════════════════════════════════════════

// POST /v1/auth/magic-link/send
#[derive(Deserialize)]
pub struct MagicLinkSendRequest {
    pub email: String,
}

pub async fn handler_request_magic_link(
    State(state): State<AppState>,
    Json(body): Json<MagicLinkSendRequest>,
) -> Result<StatusCode, ApiError> {
    let email = body.email.trim().to_lowercase();

    sqlx::query(
        "DELETE FROM auth_magic_links WHERE email = ? OR expires_at < datetime('now')"
    )
    .bind(&email)
    .execute(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let token      = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(15);

    sqlx::query(
        "INSERT INTO auth_magic_links (token, email, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&token)
    .bind(&email)
    .bind(expires_at)
    .execute(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let link = format!("{}/auth/verify?token={}", app_url, token);

    send_magic_link_email(&email, &link).await?;
    Ok(StatusCode::NO_CONTENT)
}

// POST /v1/auth/magic-link/verify
#[derive(Deserialize)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token:    String,
    pub customer: Customer,
}

pub async fn handler_verify_magic_link(
    State(state): State<AppState>,
    Json(body): Json<MagicLinkVerifyRequest>,
) -> ApiResult<AuthResponse> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT token, email FROM auth_magic_links
         WHERE token = ? AND used = 0 AND expires_at > datetime('now')"
    )
    .bind(&body.token)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::BadRequest("リンクが無効または期限切れです".into()))?;

    let email = row.1;

    sqlx::query("UPDATE auth_magic_links SET used = 1 WHERE token = ?")
        .bind(&body.token)
        .execute(&state.pool)
        .await
        .map_err(ApiError::from)?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let customer = match customer {
        Some(c) => c,
        None => {
            let new_uuid = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO customers (uuid, email) VALUES (?, ?)")
                .bind(&new_uuid).bind(&email)
                .execute(&state.pool).await.map_err(ApiError::from)?;
            sqlx::query_as::<_, Customer>(
                "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
            )
            .bind(&email).fetch_one(&state.pool).await.map_err(ApiError::from)?
        }
    };

    let token = create_customer_session(&state.pool, customer.id).await?;
    Ok(Json(AuthResponse { token, customer }))
}

// GET /v1/auth/me
pub async fn handler_get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Customer> {
    let customer = get_authenticated_customer(&state.pool, &headers).await?;
    Ok(Json(customer))
}

// POST /v1/auth/logout
pub async fn handler_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    if let Some(token) = extract_bearer(&headers) {
        sqlx::query("DELETE FROM customer_sessions WHERE token = ?")
            .bind(&token)
            .execute(&state.pool)
            .await
            .map_err(ApiError::from)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

// PATCH /v1/auth/profile
#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub email:        Option<String>,
}

pub async fn handler_update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<Customer> {
    let customer = get_authenticated_customer(&state.pool, &headers).await?;

    if let Some(name) = body.display_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query("UPDATE customers SET display_name = ? WHERE id = ?")
            .bind(name).bind(customer.id)
            .execute(&state.pool).await.map_err(ApiError::from)?;
    }
    if let Some(mail) = body.email.as_deref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
    {
        sqlx::query("UPDATE customers SET email = ? WHERE id = ?")
            .bind(mail).bind(customer.id)
            .execute(&state.pool).await.map_err(ApiError::from)?;
    }

    let updated = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE id = ?"
    )
    .bind(customer.id)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(updated))
}

// ════════════════════════════════════════════════════════════
// ハンドラー — パスキー
// ════════════════════════════════════════════════════════════

// POST /v1/auth/passkey/register/start
#[derive(Deserialize)]
pub struct PasskeyRegStartRequest {
    pub email:        Option<String>,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct PasskeyRegStartResponse {
    pub uuid:           String,
    pub challenge_json: String,
}

pub async fn handler_passkey_reg_start(
    State(state): State<AppState>,
    Json(body): Json<PasskeyRegStartRequest>,
) -> ApiResult<PasskeyRegStartResponse> {
    use webauthn_rs::prelude::Uuid as WUuid;

    let email_norm = body.email.as_deref()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty());

    let existing = if let Some(ref e) = email_norm {
        sqlx::query_as::<_, Customer>(
            "SELECT id, uuid, email, display_name FROM customers WHERE email = ?"
        )
        .bind(e).fetch_optional(&state.pool).await.map_err(ApiError::from)?
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
            .bind(&new_uuid).bind(&email_norm).bind(&body.display_name)
            .execute(&state.pool).await.map_err(ApiError::from)?;
            sqlx::query_as::<_, Customer>(
                "SELECT id, uuid, email, display_name FROM customers WHERE uuid = ?"
            )
            .bind(&new_uuid).fetch_one(&state.pool).await.map_err(ApiError::from)?
        }
    };

    let user_uuid = WUuid::parse_str(&customer.uuid)
        .map_err(|_| ApiError::Internal("UUID parse error".into()))?;
    let username = customer.email.as_deref().unwrap_or(&customer.uuid).to_string();
    let display  = customer.display_name.as_deref().unwrap_or(&username).to_string();

    let (ccr, reg_state) = state.webauthn
        .start_passkey_registration(user_uuid, &username, &display, None)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    state.pending_regs.lock().unwrap().insert(customer.uuid.clone(), reg_state);

    let challenge_json = serde_json::to_string(&ccr)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(PasskeyRegStartResponse { uuid: customer.uuid, challenge_json }))
}

// POST /v1/auth/passkey/register/finish
#[derive(Deserialize)]
pub struct PasskeyRegFinishRequest {
    pub uuid:       String,
    pub credential: serde_json::Value,
}

pub async fn handler_passkey_reg_finish(
    State(state): State<AppState>,
    Json(body): Json<PasskeyRegFinishRequest>,
) -> ApiResult<AuthResponse> {
    use webauthn_rs::prelude::RegisterPublicKeyCredential;

    let reg_state = state.pending_regs.lock().unwrap()
        .remove(&body.uuid)
        .ok_or_else(|| ApiError::BadRequest("登録セッションが見つかりません".into()))?;

    let reg_resp: RegisterPublicKeyCredential = serde_json::from_value(body.credential)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let passkey = state.webauthn
        .finish_passkey_registration(&reg_resp, &reg_state)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let public_key = serde_json::to_string(&passkey)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE uuid = ?"
    )
    .bind(&body.uuid)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    sqlx::query("INSERT INTO passkey_credentials (customer_id, public_key) VALUES (?, ?)")
        .bind(customer.id).bind(&public_key)
        .execute(&state.pool).await.map_err(ApiError::from)?;

    let token = create_customer_session(&state.pool, customer.id).await?;
    Ok(Json(AuthResponse { token, customer }))
}

// POST /v1/auth/passkey/login/start
#[derive(Serialize)]
pub struct PasskeyLoginStartResponse {
    pub session_id:     String,
    pub challenge_json: String,
}

pub async fn handler_passkey_login_start(
    State(state): State<AppState>,
) -> ApiResult<PasskeyLoginStartResponse> {
    use webauthn_rs::prelude::Passkey;

    let rows: Vec<(String,)> = sqlx::query_as("SELECT public_key FROM passkey_credentials")
        .fetch_all(&state.pool).await.map_err(ApiError::from)?;

    let passkeys: Vec<Passkey> = rows.iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    if passkeys.is_empty() {
        return Err(ApiError::BadRequest("登録済みのパスキーがありません".into()));
    }

    let (rcr, auth_state) = state.webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    state.pending_discoverable_auths.lock().unwrap()
        .insert(session_id.clone(), auth_state);

    let challenge_json = serde_json::to_string(&rcr)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(PasskeyLoginStartResponse { session_id, challenge_json }))
}

// POST /v1/auth/passkey/login/finish
#[derive(Deserialize)]
pub struct PasskeyLoginFinishRequest {
    pub session_id:  String,
    pub credential:  serde_json::Value,
}

pub async fn handler_passkey_login_finish(
    State(state): State<AppState>,
    Json(body): Json<PasskeyLoginFinishRequest>,
) -> ApiResult<AuthResponse> {
    use webauthn_rs::prelude::{PublicKeyCredential, Passkey};

    let auth_state = state.pending_discoverable_auths.lock().unwrap()
        .remove(&body.session_id)
        .ok_or_else(|| ApiError::BadRequest("認証セッションが見つかりません".into()))?;

    let auth_resp: PublicKeyCredential = serde_json::from_value(body.credential)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let auth_result = state.webauthn
        .finish_passkey_authentication(&auth_resp, &auth_state)
        .map_err(|_| ApiError::Unauthorized("パスキー認証に失敗しました".into()))?;

    let used_cred_id = auth_result.cred_id();

    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT customer_id, public_key FROM passkey_credentials"
    )
    .fetch_all(&state.pool).await.map_err(ApiError::from)?;

    let customer_id = rows.iter()
        .find_map(|(cid, json)| {
            serde_json::from_str::<Passkey>(json).ok()
                .filter(|pk| pk.cred_id() == used_cred_id)
                .map(|_| *cid)
        })
        .ok_or_else(|| ApiError::Unauthorized("顧客が見つかりません".into()))?;

    let customer = sqlx::query_as::<_, Customer>(
        "SELECT id, uuid, email, display_name FROM customers WHERE id = ?"
    )
    .bind(customer_id)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let token = create_customer_session(&state.pool, customer.id).await?;
    Ok(Json(AuthResponse { token, customer }))
}

// ════════════════════════════════════════════════════════════
// ハンドラー — スタッフ
// ════════════════════════════════════════════════════════════

// POST /v1/staff/login
#[derive(Deserialize)]
pub struct StaffLoginRequest {
    pub shop_id: i32,
    pub pin:     String,
}

#[derive(Serialize)]
pub struct StaffAuthResponse {
    pub token: String,
    pub shop:  Shop,
}

#[derive(sqlx::FromRow)]
struct ShopWithPin {
    id: i32,
    name: String,
    pin: String,
}

pub async fn handler_staff_login(
    State(state): State<AppState>,
    Json(body): Json<StaffLoginRequest>,
) -> ApiResult<StaffAuthResponse> {




    let shop_with_pin = sqlx::query_as::<_, ShopWithPin>(
        "SELECT id, name, pin FROM shops WHERE id = ?"
    )
    .bind(body.shop_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::NotFound("店舗が見つかりません".into()))?;

    let ok = bcrypt::verify(&body.pin, &shop_with_pin.pin)
        .map_err(|e| ApiError::Internal(e.to_string()))?;


    if !ok {
        return Err(ApiError::Unauthorized("PINが違います".into()));
    }

    let shop = Shop { id: shop_with_pin.id, name: shop_with_pin.name };

    let token      = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        "INSERT INTO staff_sessions (token, shop_id, expires_at) VALUES (?, ?, ?)"
    )
    .bind(&token).bind(shop.id).bind(expires_at)
    .execute(&state.pool).await.map_err(ApiError::from)?;

    Ok(Json(StaffAuthResponse { token, shop }))
}


// GET /v1/staff/me
#[derive(Serialize, sqlx::FromRow)]
pub struct StaffMeResponse {
    pub shop_id:   i32,
    pub shop_name: String,
}

pub async fn handler_staff_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<StaffMeResponse> {
    let token = extract_bearer(&headers)
        .ok_or_else(|| ApiError::Unauthorized("スタッフ認証が必要です".into()))?;

    let me = sqlx::query_as::<_, StaffMeResponse>(
        "SELECT ss.shop_id, s.name as shop_name
         FROM staff_sessions ss
         JOIN shops s ON s.id = ss.shop_id
         WHERE ss.token = ? AND ss.expires_at > datetime('now')"
    )
    .bind(&token)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::Unauthorized("スタッフセッションが無効です".into()))?;

    Ok(Json(me))
}

// POST /v1/staff/logout
pub async fn handler_staff_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    if let Some(token) = extract_bearer(&headers) {
        sqlx::query("DELETE FROM staff_sessions WHERE token = ?")
            .bind(&token)
            .execute(&state.pool)
            .await
            .map_err(ApiError::from)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

// POST /v1/staff/bottles
#[derive(Deserialize)]
pub struct RegisterBottleRequest {
    pub nfc_uid:     String,
    pub guest_name:  String,
    pub drink_name:  String,
    pub expires_days: Option<i32>,
    pub email:       Option<String>,
}

pub async fn handler_register_bottle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegisterBottleRequest>,
) -> ApiResult<Bottle> {
    let shop_id = get_authenticated_staff(&state.pool, &headers).await?;

    let now        = Utc::now();
    let expires_at = now + Duration::days(body.expires_days.unwrap_or(90) as i64);

    let result = sqlx::query(
        "INSERT INTO bottles
         (shop_id, nfc_uid, guest_name, drink_name, kept_at, expires_at, remaining_percent, email)
         VALUES (?, ?, ?, ?, ?, ?, 100, ?)"
    )
    .bind(shop_id)
    .bind(&body.nfc_uid)
    .bind(&body.guest_name)
    .bind(&body.drink_name)
    .bind(now)
    .bind(expires_at)
    .bind(&body.email)
    .execute(&state.pool)
    .await
    .map_err(ApiError::from)?;

    let bottle = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(bottle))
}

// GET /v1/staff/bottles
pub async fn handler_get_shop_bottles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<Bottle>> {
    let shop_id = get_authenticated_staff(&state.pool, &headers).await?;

    let bottles = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE shop_id = ?
         ORDER BY kept_at DESC"
    )
    .bind(shop_id)
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(bottles))
}

// PATCH /v1/staff/bottles/:id
#[derive(Deserialize)]
pub struct UpdateBottleRequest {
    pub remaining_percent: Option<i32>,
    pub drink_name:        Option<String>,
    pub expires_at:        Option<DateTime<Utc>>,
}

pub async fn handler_update_bottle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bottle_id): Path<i32>,
    Json(body): Json<UpdateBottleRequest>,
) -> ApiResult<Bottle> {
    let shop_id = get_authenticated_staff(&state.pool, &headers).await?;

    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bottles WHERE id = ? AND shop_id = ?"
    )
    .bind(bottle_id).bind(shop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    if exists == 0 {
        return Err(ApiError::NotFound("ボトルが見つかりません".into()));
    }

    if let Some(pct) = body.remaining_percent {
        sqlx::query("UPDATE bottles SET remaining_percent = ? WHERE id = ?")
            .bind(pct).bind(bottle_id)
            .execute(&state.pool).await.map_err(ApiError::from)?;
    }
    if let Some(ref name) = body.drink_name {
        sqlx::query("UPDATE bottles SET drink_name = ? WHERE id = ?")
            .bind(name).bind(bottle_id)
            .execute(&state.pool).await.map_err(ApiError::from)?;
    }
    if let Some(exp) = body.expires_at {
        sqlx::query("UPDATE bottles SET expires_at = ? WHERE id = ?")
            .bind(exp).bind(bottle_id)
            .execute(&state.pool).await.map_err(ApiError::from)?;
    }

    let bottle = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE id = ?"
    )
    .bind(bottle_id)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(bottle))
}

// ════════════════════════════════════════════════════════════
// ハンドラー — 顧客ボトル
// ════════════════════════════════════════════════════════════

// GET /v1/customer/bottles
pub async fn handler_get_my_bottles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<MyBottleItem>> {
    let customer = get_authenticated_customer(&state.pool, &headers).await?;

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
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(items))
}

// POST /v1/customer/bottles/link
#[derive(Deserialize)]
pub struct LinkBottleRequest {
    pub nfc_uid: String,
}

pub async fn handler_link_bottle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LinkBottleRequest>,
) -> ApiResult<MyBottleItem> {
    let customer = get_authenticated_customer(&state.pool, &headers).await?;

    let bottle = sqlx::query_as::<_, Bottle>(
        "SELECT id, shop_id, nfc_uid, guest_name, drink_name, remaining_percent,
                kept_at, expires_at, email
         FROM bottles WHERE nfc_uid = ?"
    )
    .bind(&body.nfc_uid)
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::NotFound(
        "このタグは未登録です。スタッフにお声がけください。".into()
    ))?;

    sqlx::query(
        "INSERT OR IGNORE INTO customer_bottles (customer_id, bottle_id) VALUES (?, ?)"
    )
    .bind(customer.id).bind(bottle.id)
    .execute(&state.pool).await.map_err(ApiError::from)?;

    let item = sqlx::query_as::<_, MyBottleItem>(
        "SELECT b.id, s.name as shop_name, b.nfc_uid, b.drink_name,
                b.remaining_percent, b.kept_at, b.expires_at
         FROM bottles b
         JOIN shops s ON s.id = b.shop_id
         WHERE b.id = ?"
    )
    .bind(bottle.id)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::from)?;

    Ok(Json(item))
}

// ════════════════════════════════════════════════════════════
// ハンドラー — AI画像解析
// ════════════════════════════════════════════════════════════

// POST /v1/staff/bottles/analyze-image
#[derive(Deserialize)]
pub struct AnalyzeImageRequest {
    pub image:      String, // base64エンコードされた画像
    pub media_type: String, // "image/jpeg" or "image/png"
}

#[derive(Serialize)]
pub struct AnalyzeImageResponse {
    pub name:        Option<String>,
    pub brand:       Option<String>,
    pub spirit_type: Option<String>,
}

pub async fn handler_analyze_bottle_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AnalyzeImageRequest>,
) -> ApiResult<AnalyzeImageResponse> {
    let _shop_id = get_authenticated_staff(&state.pool, &headers).await?;

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| ApiError::Internal("ANTHROPIC_API_KEY が設定されていません".into()))?;

    let request_body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 256,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": body.media_type,
                        "data": body.image
                    }
                },
                {
                    "type": "text",
                    "text": "このボトルのラベルを読んで、以下のJSON形式のみで答えてください:\n{\"name\":\"商品名\",\"brand\":\"ブランド名\",\"spirit_type\":\"種類（ウイスキー、焼酎、ワインなど）\"}\n不明な項目はnullにしてください。JSONのみ返してください。"
                }
            ]
        }]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let resp_json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let text = resp_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| ApiError::Internal("Claude APIの応答が不正です".into()))?;

    // マークダウンのコードブロックを除去
    let text = text.trim();
    let text = text.strip_prefix("```json").unwrap_or(text);
    let text = text.strip_prefix("```").unwrap_or(text);
    let text = text.strip_suffix("```").unwrap_or(text);
    let text = text.trim();

    let parsed: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| ApiError::Internal(format!("パース失敗: {} / レスポンス: {}", e, text)))?;

    Ok(Json(AnalyzeImageResponse {
        name:        parsed["name"].as_str().map(String::from),
        brand:       parsed["brand"].as_str().map(String::from),
        spirit_type: parsed["spirit_type"].as_str().map(String::from),
    }))
}

// ════════════════════════════════════════════════════════════
// 期限通知
// ════════════════════════════════════════════════════════════

#[derive(sqlx::FromRow)]
struct ExpiringBottle {
    guest_name:     Option<String>,
    drink_name:     Option<String>,
    email:          String,
    expires_at:     DateTime<Utc>,
    shop_name:      String,
    days_remaining: i32,
}

async fn send_expiry_email(to_email: &str, shop_name: &str, body: &str) -> Result<(), ApiError> {
    let api_key = match std::env::var("RESEND_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            tracing::info!("=== 期限通知 (dev mode) === To: {} | {}", to_email, body);
            return Ok(());
        }
    };

    let from_addr = std::env::var("RESEND_FROM")
        .unwrap_or_else(|_| "onboarding@resend.dev".to_string());

    let payload = serde_json::json!({
        "from": from_addr,
        "to": [to_email],
        "subject": format!("【{}】ボトルキープ期限のお知らせ", shop_name),
        "text": body,
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.resend.com/emails")
        .bearer_auth(&api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !res.status().is_success() {
        let msg = res.text().await.unwrap_or_default();
        return Err(ApiError::Internal(format!("Resend error: {}", msg)));
    }

    Ok(())
}

async fn notify_expiring_bottles(pool: &SqlitePool) {
    let bottles = match sqlx::query_as::<_, ExpiringBottle>(
        "SELECT b.guest_name, b.drink_name, b.email, b.expires_at, s.name as shop_name,
                CASE
                    WHEN datetime(b.expires_at) BETWEEN datetime('now', '+29 days') AND datetime('now', '+30 days') THEN 30
                    WHEN datetime(b.expires_at) BETWEEN datetime('now', '+6 days')  AND datetime('now', '+7 days')  THEN 7
                    WHEN datetime(b.expires_at) BETWEEN datetime('now', '+2 days')  AND datetime('now', '+3 days')  THEN 3
                    WHEN datetime(b.expires_at) BETWEEN datetime('now')             AND datetime('now', '+1 day')   THEN 0
                END as days_remaining
         FROM bottles b
         JOIN shops s ON s.id = b.shop_id
         WHERE b.email IS NOT NULL
         AND (
             datetime(b.expires_at) BETWEEN datetime('now', '+29 days') AND datetime('now', '+30 days')
             OR datetime(b.expires_at) BETWEEN datetime('now', '+6 days')  AND datetime('now', '+7 days')
             OR datetime(b.expires_at) BETWEEN datetime('now', '+2 days')  AND datetime('now', '+3 days')
             OR datetime(b.expires_at) BETWEEN datetime('now')             AND datetime('now', '+1 day')
         )"
    )
    .fetch_all(pool)
    .await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("期限通知クエリ失敗: {}", e);
            return;
        }
    };

    tracing::info!("期限通知対象: {}件", bottles.len());

    for bottle in bottles {
        let timing = match bottle.days_remaining {
            0  => "本日が期限です".to_string(),
            3  => "3日後に期限が迫っています".to_string(),
            7  => "1週間後に期限が迫っています".to_string(),
            30 => "1ヶ月後に期限が迫っています".to_string(),
            _  => continue,
        };

        let body = format!(
            "{}様\n\n{}の「{}」のボトルキープ期限について\n\n{}\n\n期限: {}\n\nご来店お待ちしております。",
            bottle.guest_name.as_deref().unwrap_or("お客様"),
            bottle.shop_name,
            bottle.drink_name.as_deref().unwrap_or("ボトル"),
            timing,
            bottle.expires_at.format("%Y年%m月%d日"),
        );

        if let Err(e) = send_expiry_email(&bottle.email, &bottle.shop_name, &body).await {
            tracing::error!("通知メール送信失敗 to {}: {}", bottle.email, e);
        } else {
            tracing::info!("通知メール送信完了: {} ({}日前)", bottle.email, bottle.days_remaining);
        }
    }
}

pub async fn handler_notify_test(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<serde_json::Value>, ApiError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("認証が必要です".into()))?;

    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM staff_sessions WHERE token = ?")
        .bind(token)
        .fetch_one(&state.pool)
        .await
        .map_err(ApiError::from)
        .and_then(|c| if c > 0 { Ok(()) } else { Err(ApiError::Unauthorized("無効なトークンです".into())) })?;

    notify_expiring_bottles(&state.pool).await;
    Ok(axum::Json(serde_json::json!({ "ok": true })))
}

pub async fn run_notification_loop(pool: SqlitePool) {
    let hour: u32 = std::env::var("NOTIFY_HOUR")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or(13);

    loop {
        let now = Utc::now();
        let target = now.date_naive()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
            .and_utc();

        let next_run = if now < target {
            target
        } else {
            target + Duration::days(1)
        };

        let sleep_secs = (next_run - now).num_seconds().max(0) as u64;
        tracing::info!("次の期限通知: {}秒後 ({}時)", sleep_secs, hour);

        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
        notify_expiring_bottles(&pool).await;
    }
}
