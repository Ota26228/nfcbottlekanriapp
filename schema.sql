CREATE TABLE IF NOT EXISTS shops (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS bottles (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    shop_id            INTEGER NOT NULL,
    nfc_uid            TEXT UNIQUE NOT NULL,
    guest_name         TEXT,
    drink_name         TEXT,
    remaining_percent  INTEGER DEFAULT 100,
    kept_at            DATETIME,
    expires_at         DATETIME,
    email              TEXT,
    FOREIGN KEY (shop_id) REFERENCES shops(id)
);

CREATE TABLE IF NOT EXISTS customers (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid         TEXT UNIQUE NOT NULL,
    email        TEXT UNIQUE,
    display_name TEXT,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS customer_sessions (
    token       TEXT PRIMARY KEY,
    customer_id INTEGER NOT NULL,
    expires_at  DATETIME NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);

CREATE TABLE IF NOT EXISTS customer_bottles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    bottle_id   INTEGER NOT NULL,
    linked_at   DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(id),
    FOREIGN KEY (bottle_id)   REFERENCES bottles(id),
    UNIQUE(customer_id, bottle_id)
);

CREATE TABLE IF NOT EXISTS auth_magic_links (
    token      TEXT PRIMARY KEY,
    email      TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    used       INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS passkey_credentials (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    public_key  TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);
