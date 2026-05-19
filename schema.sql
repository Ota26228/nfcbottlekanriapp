CREATE TABLE IF NOT EXISTS shops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

-- 2. ボトルテーブル（NFCのシリアル番号を主軸にする）
CREATE TABLE IF NOT EXISTS bottles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shop_id INTEGER NOT NULL,
    nfc_uid TEXT UNIQUE NOT NULL,       -- ★買ってきたタグの、元からのシリアル番号を入れる場所
    guest_name TEXT,                    -- 初回カツンまでは NULL（未登録）
    drink_name TEXT,                    -- 初回カツンまでは NULL（未登録）
    remaining_percent INTEGER DEFAULT 100,
    kept_at DATETIME,                   -- 初回登録時に日付を入れる
    expires_at DATETIME,                -- 同上（3ヶ月後など）
    email TEXT,
    FOREIGN KEY (shop_id) REFERENCES shops(id)
);
