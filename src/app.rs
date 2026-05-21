use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
// ─── HTMLシェル ──────────────────────────────────────────────

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos::hydration::{AutoReload, HydrationScripts};

    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                <script src="/nfc_bridge.js"></script>
                <script src="/passkey_bridge.js"></script>
            </head>
            <body class="bg-slate-900 text-white min-h-screen">
                <App/>
            </body>
        </html>
    }
}

// ─── ルートコンポーネント ─────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/bottlekanri.css"/>
        <Title text="ボトルキープ管理"/>

        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=path!("")              view=Home/>
                <Route path=path!("admin")         view=AdminPage/>
                <Route path=path!("shop/:shop_id") view=ShopPortal/>
                <Route path=path!("my-bottles")    view=MyBottles/>
                <Route path=path!("auth/login")    view=LoginPage/>
                <Route path=path!("auth/verify")   view=VerifyPage/>
            </Routes>
        </Router>
    }
}

// ─── ホーム ──────────────────────────────────────────────────

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen gap-6 p-6">
            <h1 class="text-3xl font-bold">"🍶 ボトルキープ"</h1>
            <p class="text-slate-400 text-center">
                "お客さんはお店のNFCタグをかざしてください。"
            </p>
            <a
                href="/my-bottles"
                class="block w-full max-w-sm bg-red-600 hover:bg-red-500
                       text-white text-center text-xl font-bold
                       py-5 rounded-2xl transition"
            >
                "マイボトルを見る"
            </a>
            <a
                href="/admin"
                class="block w-full max-w-sm bg-slate-700 hover:bg-slate-600
                       text-white text-center text-lg font-medium
                       py-4 rounded-2xl transition"
            >
                "店員管理画面"
            </a>
        </div>
    }
}

// ─── 404 ─────────────────────────────────────────────────────

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <p class="text-slate-400 text-xl">"ページが見つかりません"</p>
        </div>
    }
}

// ─── BottomSheet ─────────────────────────────────────────────

#[component]
fn BottomSheet(
    #[prop(into)] show: Signal<bool>,
    on_close: impl Fn() + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm transition-opacity duration-300"
            class=("opacity-0", move || !show.get())
            class=("pointer-events-none", move || !show.get())
            on:click=move |_| on_close()
        />
        <div
            class="fixed bottom-0 inset-x-0 z-50 bg-slate-800 rounded-t-3xl
                   shadow-2xl max-h-[88vh] overflow-y-auto
                   transition-transform duration-300 ease-out"
            class=("translate-y-full", move || !show.get())
        >
            <div class="flex justify-center pt-3 pb-1">
                <div class="w-10 h-1 bg-slate-600 rounded-full"/>
            </div>
            <div class="px-6 pb-10">
                {children()}
            </div>
        </div>
    }
}

// ─── ログインページ ───────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum LoginState {
    Input,
    PasskeyWaiting,
    MagicLinkSent,
    Error(String),
}

#[component]
fn LoginPage() -> impl IntoView {
    use leptos_router::hooks::use_query_map;

    let query = use_query_map();
    let next = move || query.get().get("next").unwrap_or_else(|| "/my-bottles".into());

    let email = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let login_state = RwSignal::new(LoginState::Input);

    // マジックリンク（緊急用）
    let send_link = Action::new(move |e: &String| {
        let e = e.clone();
        async move { crate::request_magic_link(e).await }
    });
    Effect::new(move |_| {
        if let Some(result) = send_link.value().get() {
            match result {
                Ok(_) => login_state.set(LoginState::MagicLinkSent),
                Err(e) => login_state.set(LoginState::Error(e.to_string())),
            }
        }
    });

    // パスキーログイン（discoverable: メールアドレス不要）
    let passkey_login_action = Action::new(move |(sid, resp): &(String, String)| {
        let (sid, resp) = (sid.clone(), resp.clone());
        async move { crate::passkey_discoverable_finish(sid, resp).await }
    });
    let next_c1 = next.clone();
    Effect::new(move |_| {
        if let Some(Ok(_)) = passkey_login_action.value().get() {
            let navigate = leptos_router::hooks::use_navigate();
            navigate(&next_c1(), Default::default());
        }
        if let Some(Err(e)) = passkey_login_action.value().get() {
            login_state.set(LoginState::Error(e.to_string()));
        }
    });

    // 新規パスキー登録（customer_uuid, response_json）
    let passkey_reg_action = Action::new(move |(uuid, resp): &(String, String)| {
        let (uuid, resp) = (uuid.clone(), resp.clone());
        async move { crate::passkey_first_reg_finish(uuid, resp).await }
    });
    let next_c2 = next.clone();
    Effect::new(move |_| {
        if let Some(Ok(_)) = passkey_reg_action.value().get() {
            let navigate = leptos_router::hooks::use_navigate();
            navigate(&next_c2(), Default::default());
        }
        if let Some(Err(e)) = passkey_reg_action.value().get() {
            login_state.set(LoginState::Error(e.to_string()));
        }
    });

    view! {
        <div class="min-h-screen bg-slate-900 text-white flex flex-col">
            <header class="bg-slate-800 px-4 py-4 text-center shadow">
                <h1 class="text-xl font-bold">"ボトルキープ"</h1>
            </header>

            <div class="flex-1 flex flex-col items-center justify-center p-6">
                <div class="w-full max-w-sm space-y-5">
                    {move || match login_state.get() {

                        LoginState::Input => view! {
                            <div class="space-y-4">
                                <div class="text-center space-y-2">
                                    <div class="text-5xl">"🔑"</div>
                                    <p class="font-bold text-lg">"ボトルキープ"</p>
                                </div>

                                // パスキーでログイン（メールアドレス不要）
                                <button
                                    on:click=move |_| {
                                        login_state.set(LoginState::PasskeyWaiting);
                                        #[cfg(feature = "hydrate")]
                                        {
                                            let action = passkey_login_action;
                                            wasm_bindgen_futures::spawn_local(async move {
                                                match crate::passkey_discoverable_start().await {
                                                    Ok((sid, challenge_json)) => {
                                                        match crate::passkey_call_js(
                                                            "authenticatePasskey",
                                                            &challenge_json
                                                        ).await {
                                                            Ok(resp) => { action.dispatch((sid, resp)); }
                                                            Err(err) => {
                                                                login_state.set(LoginState::Error(err));
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        login_state.set(LoginState::Error(
                                                            err.to_string()
                                                        ));
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    class="w-full bg-red-600 hover:bg-red-500 active:bg-red-700
                                           text-white text-xl font-bold py-5 rounded-2xl transition"
                                >
                                    "🔑 パスキーでログイン"
                                </button>

                                <div class="flex items-center gap-3 py-1">
                                    <div class="flex-1 h-px bg-slate-700"/>
                                    <span class="text-slate-500 text-sm">"または新規登録"</span>
                                    <div class="flex-1 h-px bg-slate-700"/>
                                </div>

                                <input
                                    type="email"
                                    placeholder="メールアドレス（任意）"
                                    prop:value=move || email.get()
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-4 text-xl
                                           focus:outline-none focus:border-red-500
                                           placeholder-slate-500"
                                />

                                <input
                                    type="text"
                                    placeholder="ニックネーム（任意）"
                                    prop:value=move || display_name.get()
                                    on:input=move |ev| display_name.set(event_target_value(&ev))
                                    class="w-full bg-slate-700 border border-slate-600
                                           rounded-xl px-4 py-3 text-base
                                           focus:outline-none focus:border-slate-400
                                           placeholder-slate-500"
                                />

                                // パスキーで新規登録（メールなしOK）
                                <button
                                    on:click=move |_| {
                                        login_state.set(LoginState::PasskeyWaiting);
                                        #[cfg(feature = "hydrate")]
                                        {
                                            let email_opt = {
                                                let e = email.get().trim().to_lowercase();
                                                if e.is_empty() { None } else { Some(e) }
                                            };
                                            let name_opt = {
                                                let n = display_name.get();
                                                let n = n.trim().to_string();
                                                if n.is_empty() { None } else { Some(n) }
                                            };
                                            let action = passkey_reg_action;
                                            wasm_bindgen_futures::spawn_local(async move {
                                                match crate::passkey_first_reg_start(
                                                    email_opt, name_opt
                                                ).await {
                                                    Ok((customer_uuid, challenge_json)) => {
                                                        match crate::passkey_call_js(
                                                            "registerPasskey",
                                                            &challenge_json
                                                        ).await {
                                                            Ok(resp) => {
                                                                action.dispatch((customer_uuid, resp));
                                                            }
                                                            Err(err) => {
                                                                login_state.set(LoginState::Error(err));
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        login_state.set(LoginState::Error(
                                                            err.to_string()
                                                        ));
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    class="w-full bg-slate-700 hover:bg-slate-600
                                           text-white text-lg font-bold py-4 rounded-2xl transition"
                                >
                                    "✨ パスキーで新規登録"
                                </button>

                                <div class="text-center pt-1">
                                    <button
                                        on:click=move |_| {
                                            let e = email.get().trim().to_string();
                                            if e.is_empty() {
                                                login_state.set(LoginState::Error(
                                                    "メールリンクにはメールアドレスが必要です".into()
                                                ));
                                                return;
                                            }
                                            send_link.dispatch(e);
                                        }
                                        class="text-slate-500 text-sm underline"
                                    >
                                        "メールリンクで登録"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),

                        LoginState::PasskeyWaiting => view! {
                            <div class="text-center space-y-4">
                                <div class="text-6xl animate-pulse">"🔑"</div>
                                <p class="text-xl font-bold">"顔認証・指紋で認証中..."</p>
                                <p class="text-slate-400 text-sm">"デバイスの認証を行ってください"</p>
                            </div>
                        }.into_any(),

                        LoginState::MagicLinkSent => view! {
                            <div class="text-center space-y-4">
                                <div class="text-6xl">"📬"</div>
                                <p class="text-2xl font-bold text-green-400">
                                    "メールを送信しました！"
                                </p>
                                <p class="text-slate-400">
                                    "メール内のリンクをタップしてログインしてください。"
                                </p>
                                <p class="text-slate-500 text-sm">"（有効期限: 15分）"</p>
                            </div>
                        }.into_any(),

                        LoginState::Error(msg) => view! {
                            <div class="space-y-4">
                                <div class="bg-red-900 border border-red-600 rounded-xl p-4">
                                    <p class="text-red-400">{msg}</p>
                                </div>
                                <button
                                    on:click=move |_| login_state.set(LoginState::Input)
                                    class="w-full bg-slate-700 text-white text-lg font-bold
                                           py-4 rounded-2xl transition"
                                >
                                    "やり直す"
                                </button>
                            </div>
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}

// ─── マジックリンク検証ページ ─────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum VerifyState {
    Verifying,
    Done,
    Error(String),
}

#[component]
fn VerifyPage() -> impl IntoView {
    use leptos_router::hooks::use_query_map;

    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default();

    let verify_state = RwSignal::new(VerifyState::Verifying);

    let verify_action = Action::new(move |t: &String| {
        let t = t.clone();
        async move { crate::verify_magic_link(t).await }
    });

    // ページロード時に自動検証
    Effect::new(move |_| {
        let t = token();
        if !t.is_empty() {
            verify_action.dispatch(t);
        } else {
            verify_state.set(VerifyState::Error("トークンがありません".into()));
        }
    });

    Effect::new(move |_| {
        if let Some(result) = verify_action.value().get() {
            match result {
                Ok(_) => verify_state.set(VerifyState::Done),
                Err(e) => verify_state.set(VerifyState::Error(e.to_string())),
            }
        }
    });

    // 成功後は /my-bottles へ遷移
    Effect::new(move |_| {
        if verify_state.get() == VerifyState::Done {
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/my-bottles", Default::default());
        }
    });

    view! {
        <div class="flex flex-col items-center justify-center min-h-screen space-y-6 p-6">
            {move || match verify_state.get() {
                VerifyState::Verifying => view! {
                    <div class="text-center space-y-4">
                        <div class="w-16 h-16 border-4 border-red-500 border-t-transparent
                                    rounded-full animate-spin mx-auto"/>
                        <p class="text-xl">"ログイン処理中..."</p>
                    </div>
                }.into_any(),

                VerifyState::Done => view! {
                    <div class="text-center space-y-4">
                        <div class="text-6xl">"✅"</div>
                        <p class="text-2xl font-bold text-green-400">"ログイン完了！"</p>
                        <p class="text-slate-400">"マイボトルへ移動します..."</p>
                    </div>
                }.into_any(),

                VerifyState::Error(msg) => view! {
                    <div class="text-center space-y-4">
                        <div class="text-6xl">"⚠️"</div>
                        <p class="text-xl font-bold text-red-400">"ログインに失敗しました"</p>
                        <p class="text-slate-500 text-sm">{msg}</p>
                        <a
                            href="/auth/login"
                            class="block bg-slate-700 text-white text-lg font-bold
                                   py-4 px-8 rounded-2xl"
                        >
                            "ログイン画面へ戻る"
                        </a>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

// ─── マイボトル一覧 ───────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum PasskeySetupState {
    Idle,
    Registering,
    Done,
    Error(String),
}

#[component]
fn MyBottles() -> impl IntoView {
    let customer = Resource::new(|| (), |_| async { crate::get_current_customer().await });
    let bottles = Resource::new(|| (), |_| async { crate::get_my_bottles().await });
    let has_passkey = Resource::new(|| (), |_| async { crate::has_passkey().await });

    let passkey_state = RwSignal::new(PasskeySetupState::Idle);

    let passkey_finish = Action::new(move |resp: &String| {
        let resp = resp.clone();
        async move { crate::passkey_register_finish(resp).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = passkey_finish.value().get() {
            passkey_state.set(PasskeySetupState::Done);
            has_passkey.refetch();
        }
        if let Some(Err(e)) = passkey_finish.value().get() {
            passkey_state.set(PasskeySetupState::Error(e.to_string()));
        }
    });

    let navigate = leptos_router::hooks::use_navigate();

    // 未ログインならログインページへ
    Effect::new(move |_| {
        if let Some(Ok(None)) = customer.get() {
            navigate("/auth/login", Default::default());
        }
    });

    let navigate2 = leptos_router::hooks::use_navigate();
    let logout_action = Action::new(move |_: &()| async { crate::logout().await });
    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            navigate2("/auth/login", Default::default());
        }
    });

    view! {
        <div class="min-h-screen bg-slate-900 text-white">
            <header class="bg-slate-800 px-4 py-4 flex items-center gap-3 shadow">
                <span class="text-2xl">"🍶"</span>
                <h1 class="text-xl font-bold">"マイボトル"</h1>
                <Suspense>
                    {move || customer.get().and_then(|r| r.ok()).flatten().map(|c| {
                        let label = c.display_name
                            .or(c.email)
                            .unwrap_or_else(|| "ゲスト".into());
                        view! {
                            <span class="text-xs text-slate-400 ml-auto">{label}</span>
                        }
                    })}
                </Suspense>
            </header>

            <div class="p-4 max-w-lg mx-auto space-y-4">
                // パスキー設定バナー（パスキー未登録のユーザーにだけ表示）
                <Suspense>
                    {move || {
                        let no_passkey = has_passkey.get()
                            .and_then(|r| r.ok())
                            .map(|v| !v)
                            .unwrap_or(false);

                        match passkey_state.get() {
                            PasskeySetupState::Idle if no_passkey => view! {
                                <div class="bg-slate-800 rounded-xl p-4 flex items-center gap-3">
                                    <span class="text-2xl">"🔑"</span>
                                    <div class="flex-1">
                                        <p class="font-medium text-sm">"パスキーを設定しますか？"</p>
                                        <p class="text-slate-500 text-xs">"次回から顔認証・指紋でログイン"</p>
                                    </div>
                                    <button
                                        on:click=move |_| {
                                            passkey_state.set(PasskeySetupState::Registering);
                                            #[cfg(feature = "hydrate")]
                                            {
                                                let finish = passkey_finish;
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    match crate::passkey_register_start().await {
                                                        Ok(challenge_json) => {
                                                            match crate::passkey_call_js(
                                                                "registerPasskey",
                                                                &challenge_json
                                                            ).await {
                                                                Ok(resp) => { finish.dispatch(resp); }
                                                                Err(e) => {
                                                                    passkey_state.set(
                                                                        PasskeySetupState::Error(e)
                                                                    );
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            passkey_state.set(
                                                                PasskeySetupState::Error(e.to_string())
                                                            );
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        class="bg-red-600 hover:bg-red-500 text-white text-sm
                                               font-bold px-4 py-2 rounded-xl transition"
                                    >
                                        "設定する"
                                    </button>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Registering => view! {
                                <div class="bg-slate-800 rounded-xl p-4 text-center">
                                    <p class="text-slate-400 animate-pulse">"顔認証・指紋を登録中..."</p>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Done => view! {
                                <div class="bg-green-900 border border-green-600 rounded-xl p-4 text-center">
                                    <p class="text-green-400 font-bold">"✅ パスキーを設定しました！"</p>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Error(msg) => view! {
                                <div class="bg-red-900 border border-red-600 rounded-xl p-4">
                                    <p class="text-red-400 text-sm">"パスキー設定失敗: "{msg}</p>
                                </div>
                            }.into_any(),
                            _ => view! { <div/> }.into_any(),
                        }
                    }}
                </Suspense>

                // ボトル一覧
                <h2 class="text-lg font-semibold text-slate-300">"キープ中のボトル"</h2>
                <Suspense fallback=move || view! {
                    <p class="text-slate-400 text-center py-8">"読み込み中..."</p>
                }>
                    {move || bottles.get().map(|result| match result {
                        Ok(list) if list.is_empty() => view! {
                            <div class="text-center py-12 space-y-3">
                                <div class="text-5xl">"🍶"</div>
                                <p class="text-slate-500">"まだボトルが登録されていません"</p>
                                <p class="text-slate-600 text-sm">
                                    "お店のNFCタグをかざしてボトルを追加しましょう"
                                </p>
                            </div>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="space-y-3">
                                {list.into_iter().map(|item| {
                                    let color = bar_color(item.remaining_percent);
                                    let width = format!("width: {}%", item.remaining_percent);
                                    let expires = fmt_date(&item.expires_at);
                                    let remaining = item.remaining_percent;
                                    view! {
                                        <div class="bg-slate-800 rounded-2xl p-4 space-y-3">
                                            <div class="flex justify-between items-start">
                                                <div>
                                                    <p class="text-xs text-slate-500">
                                                        {item.shop_name}
                                                    </p>
                                                    <p class="font-bold text-lg">
                                                        {item.drink_name.unwrap_or_else(|| "未設定".into())}
                                                    </p>
                                                </div>
                                                <span class="text-2xl font-bold text-right">
                                                    {remaining}"%"
                                                </span>
                                            </div>
                                            <div class="w-full bg-slate-700 rounded-full h-3 overflow-hidden">
                                                <div
                                                    class=format!("h-full {} rounded-full", color)
                                                    style=width
                                                />
                                            </div>
                                            <p class="text-slate-500 text-xs">"⏰ 期限: "{expires}</p>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-red-400">"エラー: "{e.to_string()}</p>
                        }.into_any(),
                    })}
                </Suspense>

                // ログアウトボタン
                <button
                    on:click=move |_| { logout_action.dispatch(()); }
                    class="w-full text-slate-500 underline text-sm py-4"
                >
                    "ログアウト"
                </button>
            </div>
        </div>
    }
}

// ─── 店舗ポータル（お客さん用）───────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum PortalState {
    Loading,
    LoggedOut,
    Idle,
    Scanning,
    Checking,
    Linked(crate::MyBottleItem),
    NotRegistered,
    Error(String),
}

fn fmt_date(dt: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    match dt {
        Some(d) => d.format("%Y年%m月%d日").to_string(),
        None => "未設定".to_string(),
    }
}

fn bar_color(pct: i32) -> &'static str {
    if pct <= 25 { "bg-red-500" }
    else if pct <= 50 { "bg-orange-500" }
    else if pct <= 75 { "bg-yellow-500" }
    else { "bg-green-500" }
}

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

    let shop_info = Resource::new(
        move || shop_id(),
        move |id| async move { crate::get_shop(id).await },
    );

    let customer_res = Resource::new(|| (), |_| async { crate::get_current_customer().await });

    let portal_state = RwSignal::new(PortalState::Loading);
    let manual_uid = RwSignal::new(String::new());

    // 認証状態に応じて初期状態を設定
    Effect::new(move |_| {
        if let Some(result) = customer_res.get() {
            match result {
                Ok(Some(_)) => {
                    if portal_state.get() == PortalState::Loading {
                        portal_state.set(PortalState::Idle);
                    }
                }
                _ => portal_state.set(PortalState::LoggedOut),
            }
        }
    });

    let link_action = Action::new(move |uid: &String| {
        let uid = uid.clone();
        async move { crate::link_bottle(uid).await }
    });

    let start_scan = move |_| {
        portal_state.set(PortalState::Scanning);

        #[cfg(feature = "hydrate")]
        {
            let nfc_available = web_sys::window()
                .and_then(|w| {
                    js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("NDEFReader")).ok()
                })
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);

            if nfc_available {
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::nfc_scan().await {
                        Ok(uid) => {
                            portal_state.set(PortalState::Checking);
                            link_action.dispatch(uid);
                        }
                        Err(e) => portal_state.set(PortalState::Error(e)),
                    }
                });
            }
        }
    };

    Effect::new(move |_| {
        if let Some(result) = link_action.value().get() {
            match result {
                Ok(item) => portal_state.set(PortalState::Linked(item)),
                Err(e) if e.to_string().contains("未登録") => {
                    portal_state.set(PortalState::NotRegistered)
                }
                Err(e) => portal_state.set(PortalState::Error(e.to_string())),
            }
        }
    });

    view! {
        <div class="min-h-screen bg-slate-900 text-white">
            <header class="bg-slate-800 px-4 py-4 flex items-center gap-3 shadow">
                <span class="text-2xl">"🍶"</span>
                <h1 class="text-xl font-bold">"ボトルキープ"</h1>
                <Suspense>
                    {move || shop_info.get().and_then(|r| r.ok()).map(|s| view! {
                        <span class="text-xs text-slate-400 ml-auto">{s.name}</span>
                    })}
                </Suspense>
            </header>

            <div class="p-4 max-w-lg mx-auto">
                {move || match portal_state.get() {

                    PortalState::Loading => view! {
                        <div class="flex justify-center items-center min-h-[80vh]">
                            <div class="w-10 h-10 border-4 border-red-500 border-t-transparent
                                        rounded-full animate-spin"/>
                        </div>
                    }.into_any(),

                    PortalState::LoggedOut => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-8">
                            <div class="text-center space-y-3">
                                <div class="text-7xl">"🍶"</div>
                                <h2 class="text-2xl font-bold">"ようこそ！"</h2>
                                <p class="text-slate-400">
                                    "ログインしてボトルを管理しましょう"
                                </p>
                            </div>
                            <a
                                href=move || format!("/auth/login?next=/shop/{}", shop_id())
                                class="block w-full max-w-sm bg-red-600 hover:bg-red-500
                                       text-white text-center text-xl font-bold
                                       py-6 rounded-2xl transition shadow-lg"
                            >
                                "ログイン / 新規登録"
                            </a>
                            <a
                                href="/my-bottles"
                                class="text-slate-400 underline text-base"
                            >
                                "マイボトルを見る"
                            </a>
                        </div>
                    }.into_any(),

                    PortalState::Idle => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-6">
                            <div class="text-center space-y-2">
                                <div class="text-7xl">"🍾"</div>
                                <h2 class="text-2xl font-bold">"このお店のボトルを追加"</h2>
                                <p class="text-slate-400">
                                    "ボトルのNFCタグをかざして\nマイボトルに追加しましょう"
                                </p>
                            </div>
                            <button
                                on:click=start_scan
                                class="w-full max-w-sm bg-red-600 hover:bg-red-500 active:bg-red-700
                                       text-white text-2xl font-bold py-8 rounded-3xl
                                       transition shadow-lg"
                            >
                                "📱 タグをかざす"
                            </button>
                            <a
                                href="/my-bottles"
                                class="text-slate-400 underline text-lg"
                            >
                                "マイボトル一覧を見る →"
                            </a>
                        </div>
                    }.into_any(),

                    PortalState::Scanning => view! {
                        <div class="flex flex-col items-center space-y-6 py-8">
                            <div class="w-32 h-32 rounded-full bg-red-600 animate-pulse
                                        flex items-center justify-center text-5xl">
                                "📱"
                            </div>
                            <p class="text-2xl font-bold text-center">
                                "ボトルのタグをかざしてください"
                            </p>
                            <div class="w-full border border-slate-600 rounded-xl p-4 space-y-3">
                                <p class="text-slate-400 text-sm">"🖥️ PC・テスト用: UIDを手動入力"</p>
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
                                        if !uid.is_empty() {
                                            manual_uid.set(String::new());
                                            portal_state.set(PortalState::Checking);
                                            link_action.dispatch(uid);
                                        }
                                    }
                                    class="w-full bg-blue-600 hover:bg-blue-500
                                           text-white text-lg font-bold py-3 rounded-xl transition"
                                >
                                    "このUIDで追加 →"
                                </button>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="text-slate-400 underline py-2"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    PortalState::Checking => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-4">
                            <div class="w-16 h-16 border-4 border-red-500 border-t-transparent
                                        rounded-full animate-spin"/>
                            <p class="text-xl">"追加中..."</p>
                        </div>
                    }.into_any(),

                    PortalState::Linked(item) => {
                        let color = bar_color(item.remaining_percent);
                        let width = format!("width: {}%", item.remaining_percent);
                        let expires = fmt_date(&item.expires_at);
                        let drink = item.drink_name.unwrap_or_else(|| "未設定".into());
                        let remaining = item.remaining_percent;
                        let shop_name = item.shop_name;

                        view! {
                            <div class="space-y-6 py-4">
                                <div class="bg-green-900 border border-green-600 rounded-xl p-4 text-center">
                                    <p class="text-green-400 font-bold text-lg">
                                        "✅ マイボトルに追加しました！"
                                    </p>
                                </div>
                                <div class="bg-slate-800 rounded-2xl p-6 space-y-4">
                                    <p class="text-xs text-slate-500">{shop_name}</p>
                                    <p class="text-2xl font-bold">{drink}</p>
                                    <div class="space-y-1">
                                        <div class="flex justify-between items-center">
                                            <span class="text-slate-400 text-sm">"残量"</span>
                                            <span class="text-xl font-bold">{remaining}"%"</span>
                                        </div>
                                        <div class="w-full bg-slate-700 rounded-full h-4 overflow-hidden">
                                            <div
                                                class=format!("h-full {} rounded-full", color)
                                                style=width
                                            />
                                        </div>
                                    </div>
                                    <p class="text-slate-500 text-sm">"⏰ 期限: "{expires}</p>
                                </div>
                                <a
                                    href="/my-bottles"
                                    class="block w-full bg-red-600 hover:bg-red-500
                                           text-white text-center text-xl font-bold
                                           py-5 rounded-2xl transition"
                                >
                                    "マイボトル一覧を見る →"
                                </a>
                                <button
                                    on:click=move |_| portal_state.set(PortalState::Idle)
                                    class="w-full text-slate-400 underline py-2"
                                >
                                    "別のボトルを追加する"
                                </button>
                            </div>
                        }.into_any()
                    },

                    PortalState::NotRegistered => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-6">
                            <div class="text-7xl">"❓"</div>
                            <div class="text-center space-y-2">
                                <p class="text-2xl font-bold">"このボトルは未登録です"</p>
                                <p class="text-slate-400">"スタッフにお声がけください"</p>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="w-full max-w-sm bg-slate-700 hover:bg-slate-600
                                       text-white text-xl font-bold py-5 rounded-2xl transition"
                            >
                                "戻る"
                            </button>
                        </div>
                    }.into_any(),

                    PortalState::Error(msg) => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-6">
                            <div class="text-7xl">"⚠️"</div>
                            <div class="text-center space-y-2">
                                <p class="text-2xl font-bold text-red-400">
                                    "もう一度試してみて！"
                                </p>
                                <p class="text-slate-500 text-sm">{msg}</p>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="w-full max-w-sm bg-slate-700 hover:bg-slate-600
                                       text-white text-xl font-bold py-5 rounded-2xl transition"
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

// ─── 管理画面（店員用）───────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
enum ScanState {
    Idle,
    Scanning,
    Form(String),
    Registering,
    Done,
    Error(String),
}

#[component]
fn AdminPage() -> impl IntoView {
    let shop_id = 1i32;

    let scan_state = RwSignal::new(ScanState::Idle);
    let guest_name = RwSignal::new(String::new());
    let drink_name = RwSignal::new(String::new());
    let expires_days = RwSignal::new(90i32);
    let manual_uid = RwSignal::new(String::new());

    let bottles = Resource::new(
        move || scan_state.get() == ScanState::Idle,
        move |_| async move { crate::get_shop_bottles(shop_id).await },
    );

    let start_scan = move |_| {
        scan_state.set(ScanState::Scanning);

        #[cfg(feature = "hydrate")]
        {
            let nfc_available = web_sys::window()
                .and_then(|w| {
                    js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("NDEFReader")).ok()
                })
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);

            if nfc_available {
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::nfc_scan().await {
                        Ok(uid) => scan_state.set(ScanState::Form(uid)),
                        Err(e) => scan_state.set(ScanState::Error(e)),
                    }
                });
            }
        }
    };

    let register_action = Action::new(
        move |(uid, name, drink, days): &(String, String, String, i32)| {
            let (uid, name, drink, days) = (uid.clone(), name.clone(), drink.clone(), *days);
            async move {
                crate::register_bottle(shop_id, uid, name, drink, days).await
            }
        },
    );

    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(_) => {
                    scan_state.set(ScanState::Done);
                    guest_name.set(String::new());
                    drink_name.set(String::new());
                }
                Err(e) => scan_state.set(ScanState::Error(e.to_string())),
            }
        }
    });

    view! {
        <div class="min-h-screen bg-slate-900 text-white">
            <header class="bg-slate-800 px-4 py-4 flex items-center gap-3 shadow">
                <span class="text-2xl">"🍶"</span>
                <h1 class="text-xl font-bold">"管理画面"</h1>
                <span class="text-xs text-slate-400 ml-auto">"デモバー"</span>
            </header>

            <div class="p-4 max-w-lg mx-auto space-y-4">
                {move || match scan_state.get() {

                    ScanState::Idle => view! {
                        <div class="space-y-4">
                            <button
                                on:click=start_scan
                                class="w-full bg-red-600 hover:bg-red-500 active:bg-red-700
                                       text-white text-2xl font-bold py-6 rounded-2xl
                                       transition shadow-lg"
                            >
                                "＋ 新規ボトル登録"
                            </button>

                            <h2 class="text-lg font-semibold text-slate-300 mt-4">"登録済みボトル"</h2>
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
                                                    <p class="text-2xl font-bold text-green-400">
                                                        {b.remaining_percent}"%"
                                                    </p>
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

                    ScanState::Scanning => view! {
                        <div class="space-y-6">
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
                            <div class="border border-slate-600 rounded-xl p-4 space-y-3">
                                <p class="text-slate-400 text-sm">"🖥️ PC・テスト用: UIDを手動入力"</p>
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

                    ScanState::Form(uid) => view! {
                        <div class="space-y-5">
                            <div class="bg-green-900 border border-green-600 rounded-xl p-3">
                                <p class="text-green-400 text-xs">"タグ読み取り完了 ✓"</p>
                                <p class="text-slate-300 text-xs font-mono mt-1">{uid.clone()}</p>
                            </div>

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

                    ScanState::Registering => view! {
                        <div class="flex flex-col items-center justify-center py-20 space-y-4">
                            <div class="w-16 h-16 border-4 border-red-500 border-t-transparent
                                        rounded-full animate-spin"/>
                            <p class="text-xl">"登録中..."</p>
                        </div>
                    }.into_any(),

                    ScanState::Done => view! {
                        <div class="flex flex-col items-center justify-center py-20 space-y-6">
                            <div class="text-7xl animate-bounce">"✅"</div>
                            <p class="text-2xl font-bold text-green-400">"登録完了！"</p>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full bg-slate-700 hover:bg-slate-600
                                       text-white text-xl font-bold py-5 rounded-2xl transition"
                            >
                                "続けて登録する"
                            </button>
                        </div>
                    }.into_any(),

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
                                       text-white text-xl font-bold py-5 rounded-2xl transition"
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
