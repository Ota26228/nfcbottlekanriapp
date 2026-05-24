# ボトルキープ管理 API ドキュメント

## 概要

- **本番URL**: `https://nfcbottlekanriapp.fly.dev/v1`
- **開発URL**: `http://localhost:3000/v1`
- リクエスト・レスポンスはすべて JSON
- 認証が必要なエンドポイントは `Authorization: Bearer <token>` ヘッダーを付ける

## トークンの種類

| 種類 | 取得方法 | 有効期限 |
|------|----------|----------|
| 顧客トークン | `/auth/magic-link/verify` または `/auth/passkey/login/finish` | 30日 |
| スタッフトークン | `/staff/login` | 7日 |

---

## 店舗

### 店舗一覧を取得
```
GET /shops
```
認証不要。ログイン画面の店舗選択ドロップダウンに使う。

**レスポンス例**
```json
[
  { "id": 1, "name": "デモバー" },
  { "id": 2, "name": "BAR LUMEN" }
]
```

### 店舗を登録
```
POST /shops
```
認証不要。新規店舗の初回セットアップ時に使う。

**リクエスト**
```json
{ "name": "BAR LUMEN", "pin": "1234" }
```

**レスポンス**
```json
{ "id": 2, "name": "BAR LUMEN" }
```

### 店舗情報を取得
```
GET /shops/{shop_id}
```
認証不要。NFCタップ後に店舗名を表示するときに使う。

---

## 顧客認証

### マジックリンクを送信
```
POST /auth/magic-link/send
```
メールにログインリンクを送る。リンクの有効期限は15分。

**リクエスト**
```json
{ "email": "user@example.com" }
```

**レスポンス**: 204 No Content

### マジックリンクでログイン
```
POST /auth/magic-link/verify
```
メール内URLのトークンを送ってログイン。新規ユーザーは自動作成。

**リクエスト**
```json
{ "token": "uuid-token-from-email" }
```

**レスポンス**
```json
{
  "token": "bearer-token",
  "customer": {
    "id": 1,
    "uuid": "...",
    "email": "user@example.com",
    "display_name": null
  }
}
```

### パスキー登録（① チャレンジ取得）
```
POST /auth/passkey/register/start
```

**リクエスト**
```json
{ "email": "user@example.com", "display_name": "田中" }
```

**レスポンス**
```json
{ "uuid": "...", "challenge_json": "..." }
```

### パスキー登録（② 完了）
```
POST /auth/passkey/register/finish
```

**リクエスト**
```json
{ "uuid": "...", "credential": { /* パスキーライブラリの戻り値 */ } }
```

**レスポンス**: AuthResponse（token + customer）

### パスキーログイン（① チャレンジ取得）
```
POST /auth/passkey/login/start
```

**レスポンス**
```json
{ "session_id": "...", "challenge_json": "..." }
```

### パスキーログイン（② 完了）
```
POST /auth/passkey/login/finish
```

**リクエスト**
```json
{ "session_id": "...", "credential": { /* パスキーライブラリの戻り値 */ } }
```

**レスポンス**: AuthResponse（token + customer）

### 顧客情報を取得
```
GET /auth/me
Authorization: Bearer <顧客トークン>
```

**レスポンス**
```json
{
  "id": 1,
  "uuid": "...",
  "email": "user@example.com",
  "display_name": "田中"
}
```

### ログアウト
```
POST /auth/logout
Authorization: Bearer <顧客トークン>
```

**レスポンス**: 204 No Content

### プロフィール更新
```
PATCH /auth/profile
Authorization: Bearer <顧客トークン>
```

**リクエスト**（省略したフィールドは変更しない）
```json
{ "display_name": "田中", "email": "new@example.com" }
```

---

## スタッフ

### スタッフログイン
```
POST /staff/login
```
店舗IDとPINでログイン。

**リクエスト**
```json
{ "shop_id": 1, "pin": "1234" }
```

**レスポンス**
```json
{
  "token": "bearer-token",
  "shop": { "id": 1, "name": "デモバー" }
}
```

### スタッフ情報を取得
```
GET /staff/me
Authorization: Bearer <スタッフトークン>
```

**レスポンス**
```json
{ "shop_id": 1, "shop_name": "デモバー" }
```

### スタッフログアウト
```
POST /staff/logout
Authorization: Bearer <スタッフトークン>
```

### ボトル一覧を取得
```
GET /staff/bottles
Authorization: Bearer <スタッフトークン>
```
自分の店舗のボトルのみ返る。

**レスポンス**
```json
[
  {
    "id": 1,
    "shop_id": 1,
    "nfc_uid": "04:A3:2B:11:FF:C2",
    "guest_name": "田中様",
    "drink_name": "山崎12年",
    "remaining_percent": 80,
    "kept_at": "2026-05-01T12:00:00Z",
    "expires_at": "2026-08-01T12:00:00Z",
    "email": "tanaka@example.com"
  }
]
```

### ボトルを登録
```
POST /staff/bottles
Authorization: Bearer <スタッフトークン>
```

**リクエスト**
```json
{
  "nfc_uid": "04:A3:2B:11:FF:C2",
  "guest_name": "田中様",
  "drink_name": "山崎12年",
  "expires_days": 90,
  "email": "tanaka@example.com",
  "remaining_pct": 100
}
```
- `expires_days`: 省略すると90日後
- `email`: 省略可（期限通知を送らない場合）

### ボトルを更新
```
PATCH /staff/bottles/{id}
Authorization: Bearer <スタッフトークン>
```

**リクエスト**（省略したフィールドは変更しない）
```json
{ "remaining_percent": 60 }
```

### ボトル画像をAI解析
```
POST /staff/bottles/analyze-image
Authorization: Bearer <スタッフトークン>
```
カメラ撮影画像をbase64で送るとラベルを読み取って返す。

**リクエスト**
```json
{
  "image": "/9j/4AAQSkZJRgAB...",
  "media_type": "image/jpeg"
}
```

**レスポンス**
```json
{
  "name": "山崎12年",
  "brand": "サントリー",
  "spirit_type": "ウイスキー"
}
```

### 期限通知テスト（開発用）
```
POST /staff/notify-test
Authorization: Bearer <スタッフトークン>
```
即座に期限通知チェックを実行してメールを送る。

---

## 顧客: マイボトル

### マイボトル一覧を取得
```
GET /customer/bottles
Authorization: Bearer <顧客トークン>
```
紐づけ済みの全店舗ボトルを返す。

**レスポンス**
```json
[
  {
    "id": 1,
    "shop_name": "デモバー",
    "nfc_uid": "04:A3:2B:11:FF:C2",
    "drink_name": "山崎12年",
    "remaining_percent": 80,
    "kept_at": "2026-05-01T12:00:00Z",
    "expires_at": "2026-08-01T12:00:00Z"
  }
]
```

### NFCタグをマイボトルに紐づける
```
POST /customer/bottles/link
Authorization: Bearer <顧客トークン>
```
お客様がNFCをかざしたときに呼ぶ。スタッフが事前に登録したUIDのみ有効。

**リクエスト**
```json
{ "nfc_uid": "04:A3:2B:11:FF:C2" }
```

---

## エラーレスポンス

すべてのエラーは以下の形式で返る。

```json
{ "error": "エラーメッセージ" }
```

| ステータス | 意味 |
|-----------|------|
| 400 | リクエストの内容が不正 |
| 401 | 認証エラー（トークンなし・期限切れ） |
| 404 | リソースが見つからない |
| 500 | サーバー内部エラー |
