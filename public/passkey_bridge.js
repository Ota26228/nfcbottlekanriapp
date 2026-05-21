// WebAuthn (パスキー) ブリッジ
// Rustのwasm-bindgen経由で window.registerPasskey / window.authenticatePasskey として呼ばれる。
// webauthn-rs のJSONフォーマット（base64url文字列）と、
// ブラウザWebAuthn API（ArrayBuffer）を相互変換する。

function base64urlToBuffer(base64url) {
    const padding = '='.repeat((4 - base64url.length % 4) % 4);
    const base64 = (base64url + padding).replace(/-/g, '+').replace(/_/g, '/');
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
}

function bufferToBase64url(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (const b of bytes) binary += String.fromCharCode(b);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

// パスキー登録: webauthn-rs の CreationChallengeResponse JSON を受け取り
// ブラウザに登録させ、RegisterPublicKeyCredential JSON を返す
window.registerPasskey = async function(optionsJson) {
    const opts = JSON.parse(optionsJson);
    const pubKey = opts.publicKey;

    pubKey.challenge = base64urlToBuffer(pubKey.challenge);
    pubKey.user.id = base64urlToBuffer(pubKey.user.id);
    if (pubKey.excludeCredentials) {
        pubKey.excludeCredentials = pubKey.excludeCredentials.map(c => ({
            ...c, id: base64urlToBuffer(c.id)
        }));
    }

    const cred = await navigator.credentials.create({ publicKey: pubKey });

    return JSON.stringify({
        id: cred.id,
        rawId: bufferToBase64url(cred.rawId),
        type: cred.type,
        response: {
            clientDataJSON:    bufferToBase64url(cred.response.clientDataJSON),
            attestationObject: bufferToBase64url(cred.response.attestationObject),
        },
    });
};

// パスキー認証: webauthn-rs の RequestChallengeResponse JSON を受け取り
// ブラウザに認証させ、PublicKeyCredential JSON を返す
window.authenticatePasskey = async function(optionsJson) {
    const opts = JSON.parse(optionsJson);
    const pubKey = opts.publicKey;

    pubKey.challenge = base64urlToBuffer(pubKey.challenge);
    if (pubKey.allowCredentials) {
        pubKey.allowCredentials = pubKey.allowCredentials.map(c => ({
            ...c, id: base64urlToBuffer(c.id)
        }));
    }

    const cred = await navigator.credentials.get({ publicKey: pubKey });

    return JSON.stringify({
        id: cred.id,
        rawId: bufferToBase64url(cred.rawId),
        type: cred.type,
        response: {
            clientDataJSON:    bufferToBase64url(cred.response.clientDataJSON),
            authenticatorData: bufferToBase64url(cred.response.authenticatorData),
            signature:         bufferToBase64url(cred.response.signature),
            userHandle: cred.response.userHandle
                ? bufferToBase64url(cred.response.userHandle) : null,
        },
    });
};
