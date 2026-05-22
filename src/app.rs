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
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>
                // PWA
                <link rel="manifest" href="/manifest.json"/>
                <meta name="theme-color" content="#F5A623"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <meta name="apple-mobile-web-app-title" content="ボトルキープ"/>
                <link rel="apple-touch-icon" href="/icons/icon.svg"/>
                <link rel="icon" type="image/svg+xml" href="/icons/icon.svg"/>
                // フォント
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
                <link href="https://fonts.googleapis.com/css2?family=DM+Serif+Display:ital@0;1&family=Plus+Jakarta+Sans:wght@400;600;700&display=swap" rel="stylesheet"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                <script src="/nfc_bridge.js"></script>
                <script src="/passkey_bridge.js"></script>
            </head>
            <body class="bg-[#0F0C16] text-[#F0EAE0] min-h-screen" style="font-family: 'Plus Jakarta Sans', sans-serif;">
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

// ─── ホーム（客専用）────────────────────────────────────────

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="relative min-h-screen bg-[#0F0C16] flex flex-col items-center justify-center overflow-hidden px-6">

            // 背景グラデーション
            <div class="absolute inset-0 pointer-events-none"
                 style="background: radial-gradient(ellipse 80% 55% at 50% -5%, rgba(245,166,35,0.18) 0%, transparent 70%), radial-gradient(ellipse 50% 40% at 80% 100%, rgba(232,85,101,0.08) 0%, transparent 60%);"/>

            // 浮遊パーティクル
            <style>
                "@keyframes floatUp {
                    0%   { transform: translateY(0) rotate(0deg); opacity: 0; }
                    10%  { opacity: 0.12; }
                    90%  { opacity: 0.07; }
                    100% { transform: translateY(-100vh) rotate(360deg); opacity: 0; }
                }
                .particle { position: absolute; animation: floatUp linear infinite; pointer-events: none; font-size: 20px; }"
            </style>
            <span class="particle" style="left:12%; bottom:-5%; animation-duration:8s; animation-delay:0s;">"🍶"</span>
            <span class="particle" style="left:62%; bottom:-5%; animation-duration:11s; animation-delay:3s;">"🍸"</span>
            <span class="particle" style="left:35%; bottom:-5%; animation-duration:7s; animation-delay:5s;">"✨"</span>
            <span class="particle" style="left:82%; bottom:-5%; animation-duration:9s; animation-delay:1.5s;">"🍷"</span>
            <span class="particle" style="left:50%; bottom:-5%; animation-duration:13s; animation-delay:7s;">"🥃"</span>

            // メインコンテンツ
            <div class="relative z-10 w-full max-w-sm flex flex-col items-center gap-8">

                // ロゴ・タイトル
                <div class="text-center">
                    <div class="text-7xl mb-4" style="filter: drop-shadow(0 0 24px rgba(245,166,35,0.4));">"🍶"</div>
                    <h1 class="text-4xl font-bold mb-1" style="font-family: 'DM Serif Display', serif; color: #F5A623;">"ボトルキープ"</h1>
                    <p class="text-xs tracking-[0.2em] text-white/30 uppercase">"Bottle Keep Manager"</p>
                </div>

                // CTAボタン群
                <div class="w-full flex flex-col gap-3">
                    <a
                        href="/my-bottles"
                        class="block w-full text-center text-xl font-bold py-6 rounded-[20px]
                               transition-transform active:scale-[0.97] text-[#1A0E00]"
                        style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 32px rgba(245,166,35,0.25);"
                    >
                        "🔑 マイボトルを見る"
                    </a>
                    <a
                        href="/shop/1"
                        class="block w-full text-center text-lg font-semibold py-4 rounded-[20px]
                               bg-white/[.06] border border-white/10 text-white/90
                               transition hover:bg-white/10 active:scale-[0.97]"
                    >
                        "📱 お店でタグをかざす"
                    </a>
                </div>

                // 説明テキスト
                <p class="text-white/30 text-sm text-center leading-relaxed">
                    "NFCタグをスマホにかざして"<br/>
                    "お気に入りのボトルを管理しよう"
                </p>
            </div>

            // 店員用隠しリンク（フッター）
            <div class="absolute bottom-5 w-full text-center">
                <a href="/admin" class="text-white/15 text-xs hover:text-white/30 transition-colors">
                    "スタッフの方はこちら"
                </a>
            </div>
        </div>
    }
}

// ─── 404 ─────────────────────────────────────────────────────

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen gap-4">
            <div class="text-6xl">"🍶"</div>
            <p class="text-white/30 text-xl">"ページが見つかりません"</p>
            <a href="/" class="text-[#F5A623] underline text-sm">"ホームへ戻る"</a>
        </div>
    }
}

// ─── BottomSheet ─────────────────────────────────────────────

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

    let magic_email  = RwSignal::new(String::new());
    let email        = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let login_state = RwSignal::new(LoginState::Input);

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
        <div class="min-h-screen bg-[#0F0C16] text-[#F0EAE0] relative overflow-hidden">

            // 背景グラデーション
            <div class="absolute inset-0 pointer-events-none"
                 style="background: radial-gradient(ellipse 70% 50% at 50% 0%, rgba(245,166,35,0.13) 0%, transparent 70%);"/>

            // ── エラートースト ────────────────────────────────────
            {move || match login_state.get() {
                LoginState::Error(msg) => view! {
                    <div class="fixed top-4 inset-x-4 z-50 max-w-sm mx-auto">
                        <div class="rounded-[18px] p-4 flex items-start gap-3"
                             style="background: rgba(232,85,101,0.12); border: 1px solid rgba(232,85,101,0.3);">
                            <p class="flex-1 text-[#FF9FAA] text-sm leading-relaxed">{msg}</p>
                            <button
                                on:click=move |_| login_state.set(LoginState::Input)
                                class="text-white/40 hover:text-white text-xl leading-none shrink-0 pt-0.5 transition-colors"
                            >"✕"</button>
                        </div>
                    </div>
                }.into_any(),
                _ => ().into_any(),
            }}

            // ── メール送信完了オーバーレイ ─────────────────────────
            {move || (login_state.get() == LoginState::MagicLinkSent).then(|| view! {
                <div class="fixed inset-0 z-40 bg-[#0F0C16] flex items-center justify-center p-6">
                    <div class="text-center space-y-5 max-w-sm">
                        <div class="text-7xl">"📬"</div>
                        <p class="text-2xl font-bold text-[#4AC894]">"メールを送信しました"</p>
                        <p class="text-white/40 leading-relaxed">
                            "メール内のリンクをタップして"<br/>
                            "ログインしてください"
                        </p>
                        <p class="text-white/25 text-sm">"有効期限：15分"</p>
                        <button
                            on:click=move |_| login_state.set(LoginState::Input)
                            class="block mx-auto mt-6 text-white/35 text-sm
                                   hover:text-white/60 underline transition-colors"
                        >"← 戻る"</button>
                    </div>
                </div>
            })}

            // ── メインフォーム ────────────────────────────────────
            <div class="relative z-10 flex flex-col items-center justify-center min-h-screen p-6">
                <div class="w-full max-w-sm space-y-6">

                    // ヘッダー
                    <div class="text-center pb-2">
                        <div class="text-5xl mb-3">"🍶"</div>
                        <h1 class="text-2xl font-bold tracking-tight mb-1"
                            style="font-family: 'DM Serif Display', serif; color: #F5A623;">
                            "BottleManagement"
                        </h1>
                        <p class="text-white/35 text-sm">"お気に入りのボトルを管理しよう"</p>
                        <p class="text-center text-white/30 text-xs">
                            "登録済みの方は指紋・顔認証でそのままログイン"
                        </p>
                    </div>



                    // ── パスキーでログイン（既存ユーザー）────────────────
                    <button
                        disabled=move || matches!(login_state.get(), LoginState::PasskeyWaiting)
                        on:click=move |_| {
                            login_state.set(LoginState::PasskeyWaiting);
                            #[cfg(feature = "hydrate")]
                            {
                                let action = passkey_login_action;
                                wasm_bindgen_futures::spawn_local(async move {
                                    match crate::passkey_discoverable_start().await {
                                        Ok((sid, challenge_json)) => {
                                            match crate::passkey_call_js("authenticatePasskey", &challenge_json).await {
                                                Ok(resp) => { action.dispatch((sid, resp)); }
                                                Err(err) => { login_state.set(LoginState::Error(err)); }
                                            }
                                        }
                                        Err(err) => { login_state.set(LoginState::Error(err.to_string())); }
                                    }
                                });
                            }
                        }
                        class="w-full text-[#1A0E00] text-xl font-bold py-6 rounded-[20px]
                               transition-all duration-150 active:scale-[0.97]
                               disabled:opacity-60 disabled:cursor-not-allowed
                               flex items-center justify-center gap-3"
                        style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 32px rgba(245,166,35,0.25);"
                    >
                        {move || if matches!(login_state.get(), LoginState::PasskeyWaiting) {
                            view! {
                                <span class="w-5 h-5 border-2 border-[#1A0E00]/30 border-t-[#1A0E00] rounded-full animate-spin inline-block shrink-0"/>
                                <span>"認証中..."</span>
                            }.into_any()
                        } else {
                            view! {
                                <span>"🔑"</span>
                                <span>"パスキーでログイン"</span>
                            }.into_any()
                        }}
                    </button>

                    // ── パスキー新規登録 ──────────────────────────────
                    <div class="rounded-[20px] p-5 space-y-3"
                         style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08);">

                        <p class="text-white/20 text-l text-center tracking-[0.12em]">"顔で新規登録"</p>
                        <input
                            type="text"
                            placeholder="ニックネーム（任意）"
                            prop:value=move || display_name.get()
                            on:input=move |ev| display_name.set(event_target_value(&ev))
                            class="w-full rounded-[14px] px-4 py-4 text-base text-[#F0EAE0]
                                   focus:outline-none placeholder-white/25"
                            style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                        />
                        <input
                            type="email"
                            placeholder="メールアドレス（任意・通知用）"
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                            class="w-full rounded-[14px] px-4 py-4 text-base text-[#F0EAE0]
                                   focus:outline-none placeholder-white/25"
                            style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                        />
                        <button
                            disabled=move || matches!(login_state.get(), LoginState::PasskeyWaiting)
                            on:click=move |_| {
                                login_state.set(LoginState::PasskeyWaiting);
                                #[cfg(feature = "hydrate")]
                                {
                                    let email_opt = { let e = email.get().trim().to_lowercase(); if e.is_empty() { None } else { Some(e) } };
                                    let name_opt  = { let n = display_name.get().trim().to_string(); if n.is_empty() { None } else { Some(n) } };
                                    let action = passkey_reg_action;
                                    wasm_bindgen_futures::spawn_local(async move {
                                        match crate::passkey_first_reg_start(email_opt, name_opt).await {
                                            Ok((uuid, challenge_json)) => {
                                                match crate::passkey_call_js("registerPasskey", &challenge_json).await {
                                                    Ok(resp) => { action.dispatch((uuid, resp)); }
                                                    Err(err) => { login_state.set(LoginState::Error(err)); }
                                                }
                                            }
                                            Err(err) => { login_state.set(LoginState::Error(err.to_string())); }
                                        }
                                    });
                                }
                            }
                            class="w-full text-[#F5A623] text-base font-bold py-4 rounded-[16px]
                                   transition-all duration-150 active:scale-[0.97]
                                   disabled:opacity-60 disabled:cursor-not-allowed"
                            style="background: rgba(245,166,35,0.12); border: 1px solid rgba(245,166,35,0.25);"
                        >
                            "✨ パスキーで新規登録"
                        </button>
                    </div>
                     // ── 区切り ────────────────────────────────────────
                    <div class="flex items-center gap-3">
                        <div class="flex-1 h-px bg-white/[.07]"/>
                        <div class="flex-1 h-px bg-white/[.07]"/>
                    </div>

                    // ── メールリンク（ログイン・登録どちらも対応）────────
                    <div class="rounded-[20px] p-5 space-y-3"
                         style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08);">
                        <p class="text-white/50 text-l text-center">"📧 メールアドレスでログイン・登録"</p>
                        <input
                            type="email"
                            placeholder="メールアドレス"
                            prop:value=move || magic_email.get()
                            on:input=move |ev| magic_email.set(event_target_value(&ev))
                            class="w-full rounded-[14px] px-4 py-4 text-base text-[#F0EAE0]
                                   focus:outline-none placeholder-white/25"
                            style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                        />
                        <button
                            on:click=move |_| {
                                let e = magic_email.get().trim().to_string();
                                if e.is_empty() {
                                    login_state.set(LoginState::Error("メールアドレスを入力してください".into()));
                                    return;
                                }
                                send_link.dispatch(e);
                            }
                            class="w-full text-[#F5A623] text-base font-bold py-4 rounded-[16px]
                                   transition-all duration-150 active:scale-[0.97]"
                            style="background: rgba(245,166,35,0.12); border: 1px solid rgba(245,166,35,0.25);"
                        >
                            "メールリンクを送る"
                        </button>
                        <p class="text-white/25 text-xs text-center">"新規登録・ログインどちらも対応"</p>
                    </div>




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

    Effect::new(move |_| {
        if verify_state.get() == VerifyState::Done {
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/my-bottles", Default::default());
        }
    });

    view! {
        <div class="flex flex-col items-center justify-center min-h-screen bg-[#0F0C16] gap-6 p-6">
            {move || match verify_state.get() {
                VerifyState::Verifying => view! {
                    <div class="text-center space-y-5">
                        <div class="w-16 h-16 border-4 border-[#F5A623]/30 border-t-[#F5A623]
                                    rounded-full animate-spin mx-auto"/>
                        <p class="text-xl text-white/70">"ログイン処理中..."</p>
                    </div>
                }.into_any(),

                VerifyState::Done => view! {
                    <div class="text-center space-y-4">
                        <div class="text-6xl">"✅"</div>
                        <p class="text-2xl font-bold text-[#4AC894]">"ログイン完了！"</p>
                        <p class="text-white/40">"マイボトルへ移動します..."</p>
                    </div>
                }.into_any(),

                VerifyState::Error(msg) => view! {
                    <div class="text-center space-y-4">
                        <div class="text-6xl">"⚠️"</div>
                        <p class="text-xl font-bold text-[#FF9FAA]">"ログインに失敗しました"</p>
                        <p class="text-white/30 text-sm">{msg}</p>
                        <a
                            href="/auth/login"
                            class="block text-[#F0EAE0] text-lg font-bold
                                   py-4 px-8 rounded-[18px] mt-2"
                            style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
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
    let bottles  = Resource::new(|| (), |_| async { crate::get_my_bottles().await });
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
        <div class="min-h-screen bg-[#0F0C16] text-[#F0EAE0]">

            // ヘッダー
            <header class="px-5 py-4 flex items-center gap-3"
                    style="background: rgba(26,21,32,0.95); border-bottom: 1px solid rgba(255,255,255,0.06); backdrop-filter: blur(12px); position: sticky; top: 0; z-index: 30;">
                <div class="w-full max-w-5xl mx-auto flex items-center gap-3">
                <span class="text-2xl">"🍶"</span>
                <h1 class="text-lg font-bold">"マイボトル"</h1>
                <Suspense>
                    {move || customer.get().and_then(|r| r.ok()).flatten().map(|c| {
                        let label = c.display_name
                            .unwrap_or_else(|| "ゲスト".into());
                        view! {
                            <div class="ml-auto flex items-center gap-2">
                                <div class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-[#F5A623]"
                                     style="background: rgba(245,166,35,0.15); border: 1px solid rgba(245,166,35,0.3);">
                                    {label.chars().next().unwrap_or('G').to_uppercase().to_string()}
                                </div>
                                <span class="text-xs text-white/40">{label}</span>
                            </div>
                        }
                    })}
                </Suspense>
                </div>
            </header>

            <div class="p-4 md:p-8 max-w-5xl mx-auto space-y-4 pb-10">

                // パスキー設定バナー
                <Suspense>
                    {move || {
                        let no_passkey = has_passkey.get()
                            .and_then(|r| r.ok())
                            .map(|v| !v)
                            .unwrap_or(false);

                        match passkey_state.get() {
                            PasskeySetupState::Idle if no_passkey => view! {
                                <div class="rounded-[18px] p-4 flex items-center gap-3"
                                     style="background: rgba(245,166,35,0.07); border: 1px solid rgba(245,166,35,0.2);">
                                    <span class="text-2xl">"🔑"</span>
                                    <div class="flex-1">
                                        <p class="font-semibold text-sm text-[#F5A623]">"パスキーを設定しますか？"</p>
                                        <p class="text-white/35 text-xs mt-0.5">"次回から顔認証・指紋でログイン"</p>
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
                                                            match crate::passkey_call_js("registerPasskey", &challenge_json).await {
                                                                Ok(resp) => { finish.dispatch(resp); }
                                                                Err(e) => { passkey_state.set(PasskeySetupState::Error(e)); }
                                                            }
                                                        }
                                                        Err(e) => { passkey_state.set(PasskeySetupState::Error(e.to_string())); }
                                                    }
                                                });
                                            }
                                        }
                                        class="text-[#F5A623] text-sm font-bold px-4 py-2 rounded-[12px] transition-colors"
                                        style="background: rgba(245,166,35,0.18); border: 1px solid rgba(245,166,35,0.35);"
                                    >
                                        "設定する"
                                    </button>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Registering => view! {
                                <div class="rounded-[18px] p-4 text-center"
                                     style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08);">
                                    <p class="text-white/50 animate-pulse text-sm">"顔認証・指紋を登録中..."</p>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Done => view! {
                                <div class="rounded-[18px] p-4 text-center"
                                     style="background: rgba(74,200,148,0.08); border: 1px solid rgba(74,200,148,0.25);">
                                    <p class="text-[#4AC894] font-bold text-sm">"✅ パスキーを設定しました！"</p>
                                </div>
                            }.into_any(),
                            PasskeySetupState::Error(msg) => view! {
                                <div class="rounded-[18px] p-4"
                                     style="background: rgba(232,85,101,0.08); border: 1px solid rgba(232,85,101,0.25);">
                                    <p class="text-[#FF9FAA] text-sm">"パスキー設定失敗: "{msg}</p>
                                </div>
                            }.into_any(),
                            _ => view! { <div/> }.into_any(),
                        }
                    }}
                </Suspense>

                // セクションラベル
                <p class="text-xs font-semibold tracking-[0.12em] text-white/30 uppercase pt-2">
                    "キープ中のボトル"
                </p>

                // ボトル一覧
                <Suspense fallback=move || view! {
                    <div class="text-center py-12 space-y-3">
                        <div class="w-10 h-10 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin mx-auto"/>
                        <p class="text-white/40 text-sm">"読み込み中..."</p>
                    </div>
                }>
                    {move || bottles.get().map(|result| match result {
                        Ok(list) if list.is_empty() => view! {
                            <div class="text-center py-16 space-y-4">
                                <div class="text-5xl opacity-40">"🍶"</div>
                                <p class="text-white/35 font-medium">"まだボトルが登録されていません"</p>
                                <p class="text-white/20 text-sm">
                                    "お店のNFCタグをかざしてボトルを追加しましょう"
                                </p>
                            </div>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {list.into_iter().map(|item| {
                                    let pct = item.remaining_percent;
                                    let bar_style = bar_gradient(pct);
                                    let pct_color = pct_text_color(pct);
                                    let pct_label = pct_badge_label(pct);
                                    let badge_style = pct_badge_style(pct);
                                    let width = format!("width: {}%; {}", pct, bar_style);
                                    let expires = fmt_date(&item.expires_at);
                                    view! {
                                        <div class="rounded-[20px] p-5 space-y-4"
                                             style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.07);">
                                            <div class="flex justify-between items-start gap-3">
                                                <div class="min-w-0">
                                                    <p class="text-xs text-white/35 mb-1">{item.shop_name}</p>
                                                    <p class="font-bold text-lg leading-tight">
                                                        {item.drink_name.unwrap_or_else(|| "未設定".into())}
                                                    </p>
                                                </div>
                                                <div class="text-right shrink-0">
                                                    <div class="text-2xl font-bold" style=format!("color: {};", pct_color)>
                                                        {pct}"%"
                                                    </div>
                                                    <span class="text-[11px] font-semibold px-2 py-0.5 rounded-full"
                                                          style=badge_style>
                                                        {pct_label}
                                                    </span>
                                                </div>
                                            </div>
                                            // プログレスバー（シマーアニメ用クラス付き）
                                            <div class="w-full h-2.5 rounded-full overflow-hidden"
                                                 style="background: rgba(255,255,255,0.08);">
                                                <div class="h-full rounded-full bottle-bar"
                                                     style=width/>
                                            </div>
                                            <p class="text-white/25 text-xs">"⏰ 期限: "{expires}</p>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="rounded-[18px] p-4"
                                 style="background: rgba(232,85,101,0.08); border: 1px solid rgba(232,85,101,0.2);">
                                <p class="text-[#FF9FAA] text-sm">"エラー: "{e.to_string()}</p>
                            </div>
                        }.into_any(),
                    })}
                </Suspense>

                // ログアウト
                <button
                    on:click=move |_| { logout_action.dispatch(()); }
                    class="w-full text-white/25 hover:text-white/50 underline text-sm py-4 transition-colors"
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

fn bar_gradient(pct: i32) -> &'static str {
    if pct <= 25 {
        "background: linear-gradient(90deg, #E85565, #FF8C42);"
    } else if pct <= 50 {
        "background: linear-gradient(90deg, #F5A623, #FFD04A);"
    } else if pct <= 75 {
        "background: linear-gradient(90deg, #F5A623, #E8D045);"
    } else {
        "background: linear-gradient(90deg, #4AC894, #7AE8C0);"
    }
}

fn pct_text_color(pct: i32) -> &'static str {
    if pct <= 25 { "#E85565" }
    else if pct <= 75 { "#F5A623" }
    else { "#4AC894" }
}

fn pct_badge_label(pct: i32) -> &'static str {
    if pct <= 25 { "残り少ない" }
    else if pct <= 50 { "なかなか" }
    else if pct <= 75 { "半分以上" }
    else { "たっぷり" }
}

fn pct_badge_style(pct: i32) -> &'static str {
    if pct <= 25 {
        "background: rgba(232,85,101,0.12); color: #E85565; border: 1px solid rgba(232,85,101,0.25);"
    } else if pct <= 75 {
        "background: rgba(245,166,35,0.12); color: #F5A623; border: 1px solid rgba(245,166,35,0.25);"
    } else {
        "background: rgba(74,200,148,0.12); color: #4AC894; border: 1px solid rgba(74,200,148,0.25);"
    }
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
    let manual_uid   = RwSignal::new(String::new());

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
                .and_then(|w| js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("NDEFReader")).ok())
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);

            if nfc_available {
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::nfc_scan().await {
                        Ok(uid) => { portal_state.set(PortalState::Checking); link_action.dispatch(uid); }
                        Err(e)  => portal_state.set(PortalState::Error(e)),
                    }
                });
            }
        }
    };

    Effect::new(move |_| {
        if let Some(result) = link_action.value().get() {
            match result {
                Ok(item) => {
                    portal_state.set(PortalState::Linked(item));
                    #[cfg(feature = "hydrate")]
                    let _ = js_sys::eval("setTimeout(() => { window.location.href = '/my-bottles' }, 2000)");
                }
                Err(e) if e.to_string().contains("未登録") => portal_state.set(PortalState::NotRegistered),
                Err(e)    => portal_state.set(PortalState::Error(e.to_string())),
            }
        }
    });

    view! {
        <div class="min-h-screen bg-[#0F0C16] text-[#F0EAE0]">

            // ヘッダー
            <header style="background: rgba(26,21,32,0.95); border-bottom: 1px solid rgba(255,255,255,0.06); backdrop-filter: blur(12px); position: sticky; top: 0; z-index: 30;">
                <div class="max-w-2xl mx-auto px-5 py-4 flex items-center gap-3">
                    <span class="text-2xl">"🍶"</span>
                    <h1 class="text-lg font-bold">"ボトルキープ"</h1>
                    <Suspense>
                        {move || shop_info.get().and_then(|r| r.ok()).map(|s| view! {
                            <span class="text-xs text-white/35 ml-auto">{s.name}</span>
                        })}
                    </Suspense>
                </div>
            </header>

            <div class="p-4 md:p-8 max-w-2xl mx-auto">
                {move || match portal_state.get() {

                    PortalState::Loading => view! {
                        <div class="flex justify-center items-center min-h-[80vh]">
                            <div class="w-10 h-10 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin"/>
                        </div>
                    }.into_any(),

                    PortalState::LoggedOut => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-8">
                            <div class="text-center space-y-3">
                                <div class="text-7xl">"🍶"</div>
                                <h2 class="text-2xl font-bold">"ようこそ！"</h2>
                                <p class="text-white/40">"ログインしてボトルを管理しましょう"</p>
                            </div>
                            <div class="w-full max-w-sm space-y-3">
                                <a
                                    href=move || format!("/auth/login?next=/shop/{}", shop_id())
                                    class="block w-full text-center text-xl font-bold py-6 rounded-[20px] text-[#1A0E00] transition active:scale-[0.97]"
                                    style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 32px rgba(245,166,35,0.2);"
                                >
                                    "ログイン / 新規登録"
                                </a>
                                <a
                                    href="/my-bottles"
                                    class="block text-center text-white/40 text-base underline"
                                >
                                    "マイボトルを見る"
                                </a>
                            </div>
                        </div>
                    }.into_any(),

                    PortalState::Idle => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-8">
                            <div class="text-center space-y-3">
                                <div class="text-7xl">"🍾"</div>
                                <h2 class="text-2xl font-bold">"このお店のボトルを追加"</h2>
                                <p class="text-white/40 text-sm leading-relaxed">
                                    "ボトルのNFCタグをかざして"<br/>
                                    "マイボトルに追加しましょう"
                                </p>
                            </div>
                            <div class="w-full max-w-sm space-y-3">
                                <button
                                    on:click=start_scan
                                    class="w-full text-[#1A0E00] text-2xl font-bold py-8 rounded-[24px]
                                           transition active:scale-[0.97]"
                                    style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 40px rgba(245,166,35,0.25);"
                                >
                                    "📱 タグをかざす"
                                </button>
                                <a
                                    href="/my-bottles"
                                    class="block text-center text-white/35 underline text-base"
                                >
                                    "マイボトル一覧を見る →"
                                </a>
                            </div>
                        </div>
                    }.into_any(),

                    PortalState::Scanning => view! {
                        <div class="flex flex-col items-center space-y-8 py-12">
                            // パルスリング
                            <div class="relative flex items-center justify-center">
                                <div class="absolute w-40 h-40 rounded-full animate-ping opacity-20"
                                     style="background: rgba(245,166,35,0.3);"/>
                                <div class="absolute w-32 h-32 rounded-full animate-pulse opacity-30"
                                     style="background: rgba(245,166,35,0.4);"/>
                                <div class="relative w-28 h-28 rounded-full flex items-center justify-center text-5xl"
                                     style="background: rgba(245,166,35,0.15); border: 2px solid rgba(245,166,35,0.4);">
                                    "📱"
                                </div>
                            </div>
                            <div class="text-center">
                                <p class="text-xl font-bold mb-2">"タグをかざしてください"</p>
                                <p class="text-white/35 text-sm">"Android Chrome専用"</p>
                            </div>

                            // 手動入力（開発・PC用）
                            <div class="w-full rounded-[18px] p-4 space-y-3"
                                 style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08);">
                                <p class="text-white/35 text-sm">"🖥️ PC・テスト用: UIDを手動入力"</p>
                                <input
                                    type="text"
                                    placeholder="例: 04:a3:2b:11:ff:c2:80"
                                    prop:value=move || manual_uid.get()
                                    on:input=move |ev| manual_uid.set(event_target_value(&ev))
                                    class="w-full rounded-[14px] px-4 py-3 text-base font-mono text-[#F0EAE0] focus:outline-none placeholder-white/20"
                                    style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
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
                                    class="w-full text-[#7EB5FF] text-lg font-bold py-4 rounded-[14px] transition"
                                    style="background: rgba(100,150,255,0.12); border: 1px solid rgba(100,150,255,0.25);"
                                >
                                    "このUIDで追加 →"
                                </button>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="text-white/30 underline py-2 hover:text-white/50 transition-colors"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    PortalState::Checking => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-5">
                            <div class="w-16 h-16 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin"/>
                            <p class="text-xl text-white/60">"追加中..."</p>
                        </div>
                    }.into_any(),

                    PortalState::Linked(item) => {
                        let pct = item.remaining_percent;
                        let bar_style = format!("width: {}%; {}", pct, bar_gradient(pct));
                        let pct_color = pct_text_color(pct);
                        let expires  = fmt_date(&item.expires_at);
                        let drink    = item.drink_name.unwrap_or_else(|| "未設定".into());
                        let shop_name = item.shop_name;

                        view! {
                            <div class="space-y-5 py-6">
                                <div class="rounded-[18px] p-5 text-center"
                                     style="background: rgba(74,200,148,0.08); border: 1px solid rgba(74,200,148,0.25);">
                                    <p class="text-4xl mb-2">"✅"</p>
                                    <p class="text-[#4AC894] font-bold text-xl">"マイボトルに追加しました！"</p>
                                    <p class="text-white/35 text-sm mt-1">"まもなく一覧へ移動します..."</p>
                                </div>

                                <div class="rounded-[20px] p-6 space-y-4"
                                     style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08);">
                                    <p class="text-xs text-white/35">{shop_name}</p>
                                    <p class="text-2xl font-bold">{drink}</p>
                                    <div class="space-y-2">
                                        <div class="flex justify-between items-center">
                                            <span class="text-white/40 text-sm">"残量"</span>
                                            <span class="text-xl font-bold" style=format!("color: {};", pct_color)>
                                                {pct}"%"
                                            </span>
                                        </div>
                                        <div class="w-full h-3 rounded-full overflow-hidden"
                                             style="background: rgba(255,255,255,0.08);">
                                            <div class="h-full rounded-full bottle-bar" style=bar_style/>
                                        </div>
                                    </div>
                                    <p class="text-white/25 text-sm">"⏰ 期限: "{expires}</p>
                                </div>

                                <a
                                    href="/my-bottles"
                                    class="block w-full text-center text-xl font-bold py-6 rounded-[20px] text-[#1A0E00] transition active:scale-[0.97]"
                                    style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%);"
                                >
                                    "マイボトル一覧を見る →"
                                </a>
                                <button
                                    on:click=move |_| portal_state.set(PortalState::Idle)
                                    class="w-full text-white/30 underline py-2 hover:text-white/50 transition-colors"
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
                                <p class="text-white/40">"スタッフにお声がけください"</p>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="w-full max-w-sm text-[#F0EAE0] text-xl font-bold py-6 rounded-[20px] transition active:scale-[0.97]"
                                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
                            >
                                "戻る"
                            </button>
                        </div>
                    }.into_any(),

                    PortalState::Error(msg) => view! {
                        <div class="flex flex-col items-center justify-center min-h-[80vh] space-y-6">
                            <div class="text-7xl">"⚠️"</div>
                            <div class="text-center space-y-2">
                                <p class="text-2xl font-bold text-[#FF9FAA]">"もう一度試してみて！"</p>
                                <p class="text-white/35 text-sm">{msg}</p>
                            </div>
                            <button
                                on:click=move |_| portal_state.set(PortalState::Idle)
                                class="w-full max-w-sm text-[#F0EAE0] text-xl font-bold py-6 rounded-[20px] transition active:scale-[0.97]"
                                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
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
    let pin_input = RwSignal::new(String::new());
    let pin_error = RwSignal::new(Option::<String>::None);

    let staff_session = Resource::new(|| (), |_| async { crate::get_staff_session().await });

    let login_action = Action::new(move |pin: &String| {
        let pin = pin.clone();
        async move { crate::staff_login(pin).await }
    });
    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(_)  => staff_session.refetch(),
                Err(e) => pin_error.set(Some(e.to_string())),
            }
        }
    });

    let logout_action = Action::new(|_: &()| async { crate::staff_logout().await });
    Effect::new(move |_| {
        if logout_action.value().get().is_some() { staff_session.refetch(); }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-[#0F0C16] flex items-center justify-center">
                <div class="w-10 h-10 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin"/>
            </div>
        }>
            {move || staff_session.get().map(|result| match result {
                Ok(Some(shop_id)) => view! { <AdminContent shop_id logout_action/> }.into_any(),
                _ => view! {
                    <div class="min-h-screen bg-[#0F0C16] text-[#F0EAE0] flex flex-col items-center justify-center px-6">
                        <div class="w-full max-w-sm space-y-8">
                            <div class="text-center space-y-2">
                                <div class="text-6xl">"⚙️"</div>
                                <h1 class="text-2xl font-bold">"スタッフログイン"</h1>
                                <p class="text-white/35 text-sm">"店舗PINコードを入力してください"</p>
                            </div>
                            {move || pin_error.get().map(|e| view! {
                                <div class="rounded-[14px] px-4 py-3 text-center text-[#FF9FAA] text-sm"
                                     style="background: rgba(255,100,100,0.08); border: 1px solid rgba(255,100,100,0.2);">
                                    {e}
                                </div>
                            })}
                            <div class="space-y-3">
                                <input
                                    type="password"
                                    inputmode="numeric"
                                    placeholder="PINコード"
                                    prop:value=move || pin_input.get()
                                    on:input=move |ev| { pin_error.set(None); pin_input.set(event_target_value(&ev)); }
                                    on:keydown=move |ev| { if ev.key() == "Enter" { let _ = login_action.dispatch(pin_input.get()); } }
                                    class="w-full rounded-[14px] px-4 py-5 text-xl text-center font-mono text-[#F0EAE0] focus:outline-none placeholder-white/20 tracking-[0.15em]"
                                    style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
                                />
                                <button
                                    on:click=move |_| { let _ = login_action.dispatch(pin_input.get()); }
                                    disabled=move || login_action.pending().get()
                                    class="w-full text-[#1A0E00] text-xl font-bold py-5 rounded-[20px] transition active:scale-[0.97] disabled:opacity-60 disabled:cursor-not-allowed"
                                    style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 32px rgba(245,166,35,0.2);"
                                >
                                    {move || if login_action.pending().get() { "確認中..." } else { "ログイン" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn AdminContent(shop_id: i32, logout_action: Action<(), Result<(), ServerFnError>>) -> impl IntoView {
    let scan_state   = RwSignal::new(ScanState::Idle);
    let guest_name   = RwSignal::new(String::new());
    let drink_name   = RwSignal::new(String::new());
    let expires_days = RwSignal::new(90i32);
    let manual_uid   = RwSignal::new(String::new());

    let bottles = Resource::new(
        move || scan_state.get() == ScanState::Idle,
        move |_| async move { crate::get_shop_bottles(shop_id).await },
    );

    let start_scan = move |_| {
        scan_state.set(ScanState::Scanning);
        #[cfg(feature = "hydrate")]
        {
            let nfc_available = web_sys::window()
                .and_then(|w| js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("NDEFReader")).ok())
                .map(|v| !v.is_undefined() && !v.is_null())
                .unwrap_or(false);

            if nfc_available {
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::nfc_scan().await {
                        Ok(uid) => scan_state.set(ScanState::Form(uid)),
                        Err(e)  => scan_state.set(ScanState::Error(e)),
                    }
                });
            }
        }
    };

    let register_action = Action::new(
        move |(uid, name, drink, days): &(String, String, String, i32)| {
            let (uid, name, drink, days) = (uid.clone(), name.clone(), drink.clone(), *days);
            async move { crate::register_bottle(shop_id, uid, name, drink, days).await }
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
        <div class="min-h-screen bg-[#0F0C16] text-[#F0EAE0]">

            // ヘッダー
            <header style="background: rgba(26,21,32,0.95); border-bottom: 1px solid rgba(255,255,255,0.06); backdrop-filter: blur(12px); position: sticky; top: 0; z-index: 30;">
                <div class="max-w-4xl mx-auto px-5 py-4 flex items-center gap-3">
                    <span class="text-2xl">"⚙️"</span>
                    <div>
                        <h1 class="text-lg font-bold leading-tight">"管理画面"</h1>
                        <p class="text-xs text-white/30 leading-none">"デモバー"</p>
                    </div>
                    <button
                        on:click=move |_| { let _ = logout_action.dispatch(()); }
                        class="ml-auto text-white/30 text-sm hover:text-white/60 transition-colors"
                    >"ログアウト"</button>
                </div>
            </header>

            <div class="p-4 md:p-8 max-w-4xl mx-auto space-y-4 pb-10">
                {move || match scan_state.get() {

                    ScanState::Idle => view! {
                        <div class="space-y-5">
                            <div class="max-w-md mx-auto">
                                <button
                                    on:click=start_scan
                                    class="w-full text-[#1A0E00] text-2xl font-bold py-7 rounded-[22px]
                                           transition active:scale-[0.97]"
                                    style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 40px rgba(245,166,35,0.25);"
                                >
                                    "＋ 新規ボトル登録"
                                </button>
                            </div>

                            <p class="text-xs font-semibold tracking-[0.12em] text-white/30 uppercase pt-2">
                                "登録済みボトル"
                            </p>

                            <Suspense fallback=move || view! {
                                <div class="text-center py-8">
                                    <div class="w-8 h-8 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin mx-auto"/>
                                </div>
                            }>
                                {move || bottles.get().map(|result| match result {
                                    Ok(list) if list.is_empty() => view! {
                                        <div class="text-center py-12">
                                            <p class="text-white/30 text-sm">"まだボトルが登録されていません"</p>
                                        </div>
                                    }.into_any(),
                                    Ok(list) => view! {
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                            {list.into_iter().map(|b| {
                                                let pct = b.remaining_percent;
                                                let pct_color = pct_text_color(pct);
                                                view! {
                                                    <div class="rounded-[18px] p-4 flex items-center justify-between gap-4"
                                                         style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.07);">
                                                        <div class="min-w-0">
                                                            <p class="font-bold text-base truncate">
                                                                {b.guest_name.unwrap_or_else(|| "未設定".into())}
                                                            </p>
                                                            <p class="text-white/40 text-sm mt-0.5 truncate">
                                                                {b.drink_name.unwrap_or_else(|| "未設定".into())}
                                                            </p>
                                                        </div>
                                                        <div class="text-2xl font-bold shrink-0"
                                                             style=format!("color: {};", pct_color)>
                                                            {pct}"%"
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <div class="rounded-[18px] p-4"
                                             style="background: rgba(232,85,101,0.08); border: 1px solid rgba(232,85,101,0.2);">
                                            <p class="text-[#FF9FAA] text-sm">"エラー: "{e.to_string()}</p>
                                        </div>
                                    }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    }.into_any(),

                    ScanState::Scanning => view! {
                        <div class="space-y-8 max-w-md mx-auto">
                            <div class="flex flex-col items-center py-10 space-y-6">
                                <div class="relative flex items-center justify-center">
                                    <div class="absolute w-36 h-36 rounded-full animate-ping opacity-15"
                                         style="background: rgba(245,166,35,0.35);"/>
                                    <div class="absolute w-28 h-28 rounded-full animate-pulse opacity-25"
                                         style="background: rgba(245,166,35,0.45);"/>
                                    <div class="relative w-24 h-24 rounded-full flex items-center justify-center text-4xl"
                                         style="background: rgba(245,166,35,0.15); border: 2px solid rgba(245,166,35,0.4);">
                                        "📱"
                                    </div>
                                </div>
                                <div class="text-center">
                                    <p class="text-xl font-bold mb-1">"タグをかざしてください"</p>
                                    <p class="text-white/30 text-sm">"Android Chrome専用"</p>
                                </div>
                            </div>

                            <div class="rounded-[18px] p-4 space-y-3"
                                 style="border: 1px solid rgba(255,255,255,0.08); background: rgba(255,255,255,0.03);">
                                <p class="text-white/35 text-sm">"🖥️ PC・テスト用"</p>
                                <input
                                    type="text"
                                    placeholder="例: 04:a3:2b:11:ff:c2:80"
                                    prop:value=move || manual_uid.get()
                                    on:input=move |ev| manual_uid.set(event_target_value(&ev))
                                    class="w-full rounded-[14px] px-4 py-3 text-base font-mono text-[#F0EAE0] focus:outline-none placeholder-white/20"
                                    style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                                />
                                <button
                                    on:click=move |_| {
                                        let uid = manual_uid.get().trim().to_string();
                                        if uid.is_empty() {
                                            scan_state.set(ScanState::Error("UIDを入力してください".into()));
                                        } else {
                                            manual_uid.set(String::new());
                                            scan_state.set(ScanState::Form(uid));
                                        }
                                    }
                                    class="w-full text-[#7EB5FF] text-lg font-bold py-4 rounded-[14px] transition"
                                    style="background: rgba(100,150,255,0.12); border: 1px solid rgba(100,150,255,0.25);"
                                >
                                    "このUIDで進む →"
                                </button>
                            </div>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-white/30 underline text-base py-2 hover:text-white/50 transition-colors"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    ScanState::Form(uid) => view! {
                        <div class="space-y-5 max-w-md mx-auto">
                            // タグ読み取り完了バナー
                            <div class="rounded-[14px] p-3"
                                 style="background: rgba(74,200,148,0.08); border: 1px solid rgba(74,200,148,0.2);">
                                <p class="text-[#4AC894] text-xs font-semibold">"タグ読み取り完了 ✓"</p>
                                <p class="text-white/40 text-xs font-mono mt-1">{uid.clone()}</p>
                            </div>

                            // お客様名
                            <div class="space-y-2">
                                <label class="block text-sm font-semibold text-white/60">"お客様のお名前"</label>
                                <input
                                    type="text"
                                    placeholder="例: 田中様"
                                    prop:value=move || guest_name.get()
                                    on:input=move |ev| guest_name.set(event_target_value(&ev))
                                    class="w-full rounded-[14px] px-4 py-5 text-xl text-[#F0EAE0] focus:outline-none placeholder-white/20"
                                    style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                                />
                            </div>

                            // 銘柄
                            <div class="space-y-2">
                                <label class="block text-sm font-semibold text-white/60">"お酒の銘柄"</label>
                                <input
                                    type="text"
                                    placeholder="例: 山崎 12年"
                                    prop:value=move || drink_name.get()
                                    on:input=move |ev| drink_name.set(event_target_value(&ev))
                                    class="w-full rounded-[14px] px-4 py-5 text-xl text-[#F0EAE0] focus:outline-none placeholder-white/20"
                                    style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                                />
                            </div>

                            // キープ期限
                            <div class="space-y-2">
                                <label class="block text-sm font-semibold text-white/60">"キープ期限"</label>
                                <select
                                    prop:value=move || expires_days.get().to_string()
                                    on:change=move |ev| {
                                        let v: i32 = event_target_value(&ev).parse().unwrap_or(90);
                                        expires_days.set(v);
                                    }
                                    class="w-full rounded-[14px] px-4 py-5 text-xl text-[#F0EAE0] focus:outline-none"
                                    style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
                                >
                                    <option value="30">"1ヶ月"</option>
                                    <option value="60">"2ヶ月"</option>
                                    <option value="90" selected>"3ヶ月（標準）"</option>
                                    <option value="180">"6ヶ月"</option>
                                </select>
                            </div>

                            <button
                                on:click=move |_| {
                                    let name  = guest_name.get();
                                    let drink = drink_name.get();
                                    if name.is_empty() || drink.is_empty() {
                                        scan_state.set(ScanState::Error("お名前と銘柄を入力してください".into()));
                                        return;
                                    }
                                    scan_state.set(ScanState::Registering);
                                    register_action.dispatch((uid.clone(), name, drink, expires_days.get()));
                                }
                                class="w-full text-[#1A0E00] text-2xl font-bold py-5 rounded-[20px]
                                       transition active:scale-[0.97]"
                                style="background: linear-gradient(135deg, #F5A623 0%, #E8572A 100%); box-shadow: 0 8px 32px rgba(245,166,35,0.2);"
                            >
                                "登録する"
                            </button>

                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-white/30 underline text-lg py-2 hover:text-white/50 transition-colors"
                            >
                                "キャンセル"
                            </button>
                        </div>
                    }.into_any(),

                    ScanState::Registering => view! {
                        <div class="flex flex-col items-center justify-center py-24 space-y-5">
                            <div class="w-16 h-16 border-4 border-[#F5A623]/30 border-t-[#F5A623] rounded-full animate-spin"/>
                            <p class="text-xl text-white/60">"登録中..."</p>
                        </div>
                    }.into_any(),

                    ScanState::Done => view! {
                        <div class="flex flex-col items-center justify-center py-24 space-y-6">
                            <div class="text-7xl animate-bounce">"✅"</div>
                            <p class="text-2xl font-bold text-[#4AC894]">"登録完了！"</p>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-[#F0EAE0] text-xl font-bold py-6 rounded-[20px] transition active:scale-[0.97]"
                                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
                            >
                                "続けて登録する"
                            </button>
                        </div>
                    }.into_any(),

                    ScanState::Error(msg) => view! {
                        <div class="flex flex-col items-center justify-center py-24 space-y-6">
                            <div class="text-6xl">"⚠️"</div>
                            <div class="text-center">
                                <p class="text-2xl font-bold text-[#FF9FAA]">"もう一度かざしてみて！"</p>
                                <p class="text-white/35 text-sm mt-2">{msg}</p>
                            </div>
                            <button
                                on:click=move |_| scan_state.set(ScanState::Idle)
                                class="w-full text-[#F0EAE0] text-xl font-bold py-6 rounded-[20px] transition active:scale-[0.97]"
                                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1);"
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
