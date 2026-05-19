// main.rs はSSRビルド時のみコンパイルされる（cfg(feature = "ssr")）
// WASMビルド時は下の空のmain()が代わりに使われる

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use bottlekanri::app::*;

    // ─── DBセットアップ ───────────────────────────────────────
    // sqlite:./bottles.db?mode=rwc
    //   rwc = read/write/create（ファイルがなければ自動作成）
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./bottles.db?mode=rwc".to_string());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("DBへの接続に失敗しました");

    // schema.sql を実行してテーブルを作成
    // CREATE TABLE IF NOT EXISTS なので何度実行しても安全
    sqlx::query(include_str!("../schema.sql"))
        .execute(&pool)
        .await
        .expect("スキーマの作成に失敗しました");

    // デモ用の店舗データを投入（なければ作成）
    sqlx::query("INSERT OR IGNORE INTO shops (id, name) VALUES (1, 'デモバー')")
        .execute(&pool)
        .await
        .expect("デモデータの作成に失敗しました");

    log!("DB初期化完了");

    // ─── Leptosルーター設定 ───────────────────────────────────
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // leptos_routes_with_context: サーバー関数の中で use_context::<SqlitePool>()
    // が使えるように、DBプールをLeptosのコンテキストに注入する
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                // このクロージャは各リクエスト処理の開始時に呼ばれ、
                // Leptosのリアクティブコンテキストにpool を登録する
                let pool = pool.clone();
                move || provide_context(pool.clone())
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

// WASMビルド時はこちらが使われる（実際には hydrate() が呼ばれる）
#[cfg(not(feature = "ssr"))]
pub fn main() {}
