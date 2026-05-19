use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

// ─── HTMLシェル ──────────────────────────────────────────────
//
// サーバーがHTMLをレスポンスするときの「外枠」。
// <head>のスクリプトやCSS、<body>のApp()マウント場所を定義する。
// ssr featureのときだけコンパイルされる。
#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos::hydration::{AutoReload, HydrationScripts};

    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // AutoReload: 開発中にファイルが変わったとき自動リロードするスクリプト
                <AutoReload options=options.clone()/>
                // HydrationScripts: WASMバンドルを読み込むscriptタグを自動生成する
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                // NFCブリッジJS: window.scanNfcTag() を定義する
                <script src="/nfc_bridge.js"></script>
            </head>
            <body class="bg-slate-900 text-white min-h-screen">
                <App/>
            </body>
        </html>
    }
}

// ─── ルートコンポーネント ─────────────────────────────────────
//
// #[component] マクロ: Leptos のコンポーネント（React でいう function component）
// pub fn App() -> impl IntoView: 画面に表示するHTMLを返す関数

#[component]
pub fn App() -> impl IntoView {
    // メタタグのコンテキストを初期化（Title, Stylesheetなどを使うために必要）
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/bottlekanri.css"/>
        <Title text="ボトルキープ管理"/>

        <Router>
            // ここでURLに対応するコンポーネントを定義する
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=path!("")       view=Home/>
                <Route path=path!("admin")  view=AdminPage/>
                <Route path=path!("shop/:shop_id") view=ShopPortal/>
            </Routes>
        </Router>
    }
}

// ─── ホーム ──────────────────────────────────────────────────

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen gap-6 p-6">
            <h1 class="text-3xl font-bold">{"🍶 ボトルキープ"}</h1>
            <p class="text-slate-400 text-center">
                "店員の方は管理画面へ。お客さんはお店のポータルNFCをかざしてください。"
            </p>
            <a
                href="/admin"
                class="block w-full max-w-sm bg-red-600 hover:bg-red-500
                       text-white text-center text-xl font-bold
                       py-5 rounded-2xl transition"
            >
                "店員管理画面へ"
            </a>
        </div>
    }
}

// ─── 404ページ ────────────────────────────────────────────────

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <p class="text-slate-400 text-xl">"ページが見つかりません"</p>
        </div>
    }
}

// ─── 管理画面（店員用）───────────────────────────────────────
//
// NFCタグのUID読み取り → ボトル情報入力 → DB登録 というフローを担う

// NFC読み取り〜登録フローの状態
// Leptosのシグナル（後述）で管理する
// allow(dead_code): Form と Error は hydrate ビルドでのみ生成されるが
// SSR ビルドからは見えないので dead_code 警告を抑制する
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum ScanState {
    Idle,
    Scanning,
    Form(String),   // 読み取ったUID
    Registering,
    Done,
    Error(String),
}

#[component]
fn AdminPage() -> impl IntoView {
    // ハッカソンではshop_id=1固定（デモバー）
    let shop_id = 1i32;

    // ─── シグナル（Leptosのリアクティブな状態管理）
    //
    // RwSignal<T>: 読み書き両方できるシグナル
    //   .get()  → 値を読む（呼んだコンポーネントが自動的に再レンダリングを購読）
    //   .set(v) → 値を書く（購読しているコンポーネントが自動更新）
    let scan_state = RwSignal::new(ScanState::Idle);
    let guest_name = RwSignal::new(String::new());
    let drink_name = RwSignal::new(String::new());
    let expires_days = RwSignal::new(90i32);
    // PC/テスト用の手動UID入力欄
    let manual_uid = RwSignal::new(String::new());

    // ─── ボトル一覧（Resource）
    //
    // Resource: 非同期データ取得を管理する。
    //   第1引数: ソース（依存するシグナル）。この値が変わると再取得する。
    //   第2引数: フェッチャー（非同期関数）。
    // scan_state が Idle に戻ったら（登録完了後）リストを再取得する。
    let bottles = Resource::new(
        move || scan_state.get() == ScanState::Idle,
        move |_| async move { crate::get_shop_bottles(shop_id).await },
    );

    // ─── NFCスキャン開始
    //
    // クロージャをイベントハンドラとして渡す。
    // move |_| の _ はクリックイベント（使わないので_で無視）
    let start_scan = move |_| {
        scan_state.set(ScanState::Scanning);

        #[cfg(feature = "hydrate")]
        {
            // window に NDEFReader があるか確認してNFC対応環境かどうか判定
            let nfc_available = web_sys::window()
                .and_then(|w| {
                    js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("NDEFReader")).ok()
                })
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);

            if nfc_available {
                // Android Chrome など NFC 対応環境: バックグラウンドでスキャン開始
                // タグをかざすと nfc_scan() が uid を返し Form 状態に遷移する
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::nfc_scan().await {
                        Ok(uid) => scan_state.set(ScanState::Form(uid)),
                        Err(e) => scan_state.set(ScanState::Error(e)),
                    }
                });
            }
            // NFC 非対応環境（PC 等）: Scanning 状態のままにしておく
            // → Scanning 画面に表示される手動入力フォームで対応
        }
    };

    // ─── 登録実行（Action）
    //
    // Action: サーバー関数の呼び出しを管理するLeptosの仕組み。
    //   .dispatch(input): アクションを実行する
    //   .pending():       実行中かどうかの ReadSignal<bool>
    //   .value():         結果の Signal<Option<Result<...>>>
    let register_action = Action::new(
        move |(uid, name, drink, days): &(String, String, String, i32)| {
            let (uid, name, drink, days) = (uid.clone(), name.clone(), drink.clone(), *days);
            async move {
                crate::register_bottle(shop_id, uid, name, drink, days).await
            }
        },
    );

    // アクションの結果を監視して状態を更新する
    // Effect::new: シグナルの変化に反応して実行される副作用
    //   引数の |_| は「前回の値」（最初はNone、今回は使わないので_）
    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(_) => {
                    scan_state.set(ScanState::Done);
                    // フォームをリセット
                    guest_name.set(String::new());
                    drink_name.set(String::new());
                }
                Err(e) => scan_state.set(ScanState::Error(e.to_string())),
            }
        }
    });

    // ─── ビュー
    //
    // view! マクロ: JSX風のHTMLを書く。RustのHtmlElementに変換される。
    // move || { ... }: シグナルを読む箇所はクロージャで包む（リアクティビティのため）
    view! {
        <div class="min-h-screen bg-slate-900 text-white">
            // ヘッダー
            <header class="bg-slate-800 px-4 py-4 flex items-center gap-3 shadow">
                <span class="text-2xl">"🍶"</span>
                <h1 class="text-xl font-bold">"管理画面"</h1>
                <span class="text-xs text-slate-400 ml-auto">"デモバー"</span>
            </header>

            <div class="p-4 max-w-lg mx-auto space-y-4">

                // ─── メインコンテンツ（状態によって切り替わる）
                // move || で包んでいるのは、scan_state が変わるたびに再描画させるため
                {move || match scan_state.get() {

                    // ── 待機中: 新規登録ボタン + ボトル一覧 ──
                    ScanState::Idle => view! {
                        <div class="space-y-4">
                            // 新規登録ボタン（大きく・目立つ）
                            <button
                                on:click=start_scan
                                class="w-full bg-red-600 hover:bg-red-500 active:bg-red-700
                                       text-white text-2xl font-bold py-6 rounded-2xl
                                       transition shadow-lg"
                            >
                                "＋ 新規ボトル登録"
                            </button>

                            // ボトル一覧
                            <h2 class="text-lg font-semibold text-slate-300 mt-4">"登録済みボトル"</h2>
                            // Suspense: Resourceのロード中にフォールバックを表示する
                            <Suspense fallback=move || view! {
                                <p class="text-slate-400 text-center py-4">"読み込み中..."</p>
                            }>
                                {move || bottles.get().map(|result| match result {
                                    Ok(list) if list.is_empty() => view! {
                                        <p class="text-slate-500 text-center py-8">
                                            "まだボトルが登録されていません"
                                        </p>
                                    }.into_any(),
                                    Ok(list) => view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|b| view! {
                                                <div class="bg-slate-800 rounded-xl p-4 flex items-center justify-between">
                                                    <div>
                                                        <p class="font-bold text-lg">
                                                            {b.guest_name.unwrap_or_else(|| "未設定".into())}
                                                        </p>
                                                        <p class="text-slate-400 text-sm">
                                                            {b.drink_name.unwrap_or_else(|| "未設定".into())}
                                                        </p>
                                                    </div>
                                                    // 残量バー
                                                    <div class="text-right">
                                                        <p class="text-2xl font-bold text-green-400">
                                                            {b.remaining_percent}"%"
                                                        </p>
                                                    </div>
                                                </div>
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <p class="text-red-400">"エラー: "{e.to_string()}</p>
                                    }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    }.into_any(),

                    // ── NFCスキャン中 ──
                    // Android Chrome: タグをかざすと自動でForm状態に遷移
                    // PC/Chromium:    手動入力フォームでUIDを入力して進む
                    ScanState::Scanning => view! {
                        <div class="space-y-6">
                            // NFC待機アニメーション
                            <div class="flex flex-col items-center py-8 space-y-4">
                                <div class="w-24 h-24 rounded-full bg-red-600 animate-pulse
                                            flex items-center justify-center text-4xl">
                                    "📱"
                                </div>
                                <p class="text-xl font-bold text-center">
                                    "ボトルのタグをかざしてください"
                                </p>
                                <p class="text-slate-500 text-sm text-center">
                                    "（Android Chrome専用）"
                                </p>
                            </div>

                            // PC/テスト用の手動入力セクション
                            <div class="border border-slate-600 rounded-xl p-4 space-y-3">
                                <p class="text-slate-400 text-sm">
                                    "🖥️ PC・テスト用: UIDを手動入力"
                                </p>
                                <input
                                    type="text"
                                    placeholder="例: 04:a3:2b:11:ff:c2:80"
                                    prop:value=move || manual_uid.get()
                                    on:input=move |ev| manual_uid.set(event_target_value(&ev))
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-3 text-base font-mono
                                           focus:outline-none focus:border-blue-500
                                           placeholder-slate-500"
                                />
                                <button
                                    on:click=move |_| {
                                        let uid = manual_uid.get().trim().to_string();
                                        if uid.is_empty() {
                                            scan_state.set(ScanState::Error(
                                                "UIDを入力してください".into()
                                            ));
                                        } else {
                                            manual_uid.set(String::new());
                                            scan_state.set(ScanState::Form(uid));
                                        }
                                    }
                                    class="w-full bg-blue-600 hover:bg-blue-500 active:bg-blue-700
                                           text-white text-lg font-bold py-3 rounded-xl transition"
                                >
                                    "このUIDで進む →"
                                </button>
                            </div>

                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-slate-400 underline text-base py-2"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    // ── フォーム: UID取得済み、ボトル情報を入力 ──
                    ScanState::Form(uid) => view! {
                        <div class="space-y-5">
                            <div class="bg-green-900 border border-green-600 rounded-xl p-3">
                                <p class="text-green-400 text-xs">"タグ読み取り完了 ✓"</p>
                                <p class="text-slate-300 text-xs font-mono mt-1">{uid.clone()}</p>
                            </div>

                            // お客様名入力
                            // input:value と on:input でRustのシグナルと双方向バインドする
                            <div class="space-y-2">
                                <label class="block text-lg font-medium">"お客様のお名前"</label>
                                <input
                                    type="text"
                                    placeholder="例: 田中様"
                                    prop:value=move || guest_name.get()
                                    on:input=move |ev| guest_name.set(event_target_value(&ev))
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-4 text-xl focus:outline-none
                                           focus:border-red-500 placeholder-slate-500"
                                />
                            </div>

                            // お酒の銘柄入力
                            <div class="space-y-2">
                                <label class="block text-lg font-medium">"お酒の銘柄"</label>
                                <input
                                    type="text"
                                    placeholder="例: 山崎12年"
                                    prop:value=move || drink_name.get()
                                    on:input=move |ev| drink_name.set(event_target_value(&ev))
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-4 text-xl focus:outline-none
                                           focus:border-red-500 placeholder-slate-500"
                                />
                            </div>

                            // キープ期限（セレクト）
                            <div class="space-y-2">
                                <label class="block text-lg font-medium">"キープ期限"</label>
                                <select
                                    prop:value=move || expires_days.get().to_string()
                                    on:change=move |ev| {
                                        let v: i32 = event_target_value(&ev).parse().unwrap_or(90);
                                        expires_days.set(v);
                                    }
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-4 text-xl focus:outline-none"
                                >
                                    <option value="30">"1ヶ月"</option>
                                    <option value="60">"2ヶ月"</option>
                                    <option value="90" selected>"3ヶ月（標準）"</option>
                                    <option value="180">"6ヶ月"</option>
                                </select>
                            </div>

                            // 登録ボタン
                            <button
                                on:click=move |_| {
                                    let name = guest_name.get();
                                    let drink = drink_name.get();
                                    if name.is_empty() || drink.is_empty() {
                                        scan_state.set(ScanState::Error(
                                            "お名前と銘柄を入力してください".into()
                                        ));
                                        return;
                                    }
                                    scan_state.set(ScanState::Registering);
                                    register_action.dispatch((
                                        uid.clone(),
                                        name,
                                        drink,
                                        expires_days.get(),
                                    ));
                                }
                                class="w-full bg-red-600 hover:bg-red-500 active:bg-red-700
                                       text-white text-2xl font-bold py-5 rounded-2xl
                                       transition shadow-lg"
                            >
                                "登録する"
                            </button>

                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-slate-400 underline text-lg py-2"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    // ── 登録中 ──
                    ScanState::Registering => view! {
                        <div class="flex flex-col items-center justify-center py-20 space-y-4">
                            <div class="w-16 h-16 border-4 border-red-500 border-t-transparent
                                        rounded-full animate-spin"/>
                            <p class="text-xl">"登録中..."</p>
                        </div>
                    }.into_any(),

                    // ── 登録完了 ──
                    ScanState::Done => view! {
                        <div class="flex flex-col items-center justify-center py-20 space-y-6">
                            <div class="text-7xl animate-bounce">"✅"</div>
                            <p class="text-2xl font-bold text-green-400">"登録完了！"</p>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full bg-slate-700 hover:bg-slate-600
                                       text-white text-xl font-bold py-5 rounded-2xl
                                       transition"
                            >
                                "続けて登録する"
                            </button>
                        </div>
                    }.into_any(),

                    // ── エラー ──
                    ScanState::Error(msg) => view! {
                        <div class="flex flex-col items-center justify-center py-20 space-y-6">
                            <div class="text-6xl">"⚠️"</div>
                            <p class="text-2xl font-bold text-red-400 text-center">
                                "もう一度かざしてみて！"
                            </p>
                            <p class="text-slate-400 text-sm text-center">{msg}</p>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full bg-slate-700 hover:bg-slate-600
                                       text-white text-xl font-bold py-5 rounded-2xl
                                       transition"
                            >
                                "戻る"
                            </button>
                        </div>
                    }.into_any(),
                }}

            </div>
        </div>
    }
}

// ─── 店舗ポータル（お客さん用）───────────────────────────────
// TODO Phase 1b で実装

#[component]
fn ShopPortal() -> impl IntoView {
    use leptos_router::hooks::use_params_map;

    let params = use_params_map();
    let shop_id = move || {
        params.get()
            .get("shop_id")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    };

    view! {
        <div class="flex flex-col items-center justify-center min-h-screen gap-6 p-6">
            <h1 class="text-3xl font-bold">"🍶 ボトルキープ"</h1>
            <p class="text-slate-400">"店舗ID: "{move || shop_id()}</p>
            <p class="text-slate-500 text-sm">"お客さん用ポータル（実装予定）"</p>
        </div>
    }
}
