pub mod app;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use leptos::prelude::*;

// ─── データ型定義 ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Shop {
    pub id: i32,
    pub name: String,
}

// cfg_attr: SSRビルド時だけ sqlx::FromRow を derive する
// WASMビルド時はsqlxが存在しないのでエラーになるため
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

// NFCタグ照合の結果
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BottleCheckResult {
    NotRegistered { nfc_uid: String },
    Registered(Bottle),
}

// ─── SSR専用: DBプール ────────────────────────────────────────

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::SqlitePool;
    use leptos::prelude::provide_context;

    pub fn register_db_pool(pool: SqlitePool) {
        provide_context(pool);
    }
}

// ─── WASMエントリポイント ─────────────────────────────────────

// cargo-leptos がWASMをビルドするとき、この関数を呼び出してアプリを起動する
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    // パニック時にブラウザのコンソールにわかりやすいエラーを出す
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

// ─── クライアント専用: NFC読み取り ───────────────────────────

// #[cfg(feature = "hydrate")]: WASMビルド時のみコンパイルされる
// JavaScriptの window.scanNfcTag() を Rust から呼び出す
#[cfg(feature = "hydrate")]
pub async fn nfc_scan() -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    // ブラウザの window オブジェクトを取得
    let window = web_sys::window()
        .ok_or_else(|| "windowが見つかりません".to_string())?;

    // window.scanNfcTag を取得して Function として扱う
    let func = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("scanNfcTag"))
        .map_err(|_| "nfc_bridge.jsが読み込まれていません".to_string())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "scanNfcTagが関数ではありません".to_string())?;

    // scanNfcTag() を呼び出す（戻り値はPromise）
    let promise = func
        .call0(&wasm_bindgen::JsValue::UNDEFINED)
        .map_err(|e| e.as_string().unwrap_or("呼び出しエラー".into()))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "Promiseが返されませんでした".to_string())?;

    // Promise を Rust の Future に変換して await する
    JsFuture::from(promise)
        .await
        .map(|v| v.as_string().unwrap_or_default())
        .map_err(|e| e.as_string().unwrap_or("NFCエラー".into()))
}

// ─── サーバー関数 ─────────────────────────────────────────────
//
// #[server] マクロ: この関数はサーバー側でのみ実行される。
// WASMからこの関数を呼ぶと、自動的に /api/... へHTTPリクエストになる。

// NFCタグ照合（既存）
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

// 新規ボトル登録（店員用）
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

    // INSERT して最後に挿入したIDを取得
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

    // 挿入したレコードをSELECTして返す
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

// 店舗のボトル一覧取得（店員用）
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
