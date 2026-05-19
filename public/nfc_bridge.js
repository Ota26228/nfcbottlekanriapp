// NFCタグのUIDを読み取るブリッジ関数
// Android Chrome専用。RustのWASMコードから window.scanNfcTag() として呼び出される。
//
// Web NFC APIの仕組み:
//   NDEFReader という標準APIを使う。
//   .scan() を呼ぶと「NFC待機状態」になり、タグをかざすと onreading イベントが発火する。
//   event.serialNumber がタグのハードウェアUID（例: "04:a3:2b:11:ff:c2:80"）

window.scanNfcTag = function () {
    return new Promise((resolve, reject) => {
        // NDEFReader がなければこの端末はWeb NFC非対応
        if (!("NDEFReader" in window)) {
            reject("この端末はNFCに対応していません（Android Chrome専用）");
            return;
        }

        const reader = new NDEFReader();

        reader.scan()
            .then(() => {
                // scan() が成功 → タグ待ち状態になった
                reader.onreading = (event) => {
                    // serialNumber = ハードウェアUID（これがボトルの"名札"になる）
                    resolve(event.serialNumber);
                };
                reader.onerror = (event) => {
                    reject("読み取りエラー: " + event.message);
                };
            })
            .catch((err) => {
                // よくある原因: HTTPSでない、ユーザー操作なしで呼んだ、権限が拒否された
                reject("NFCスキャン開始エラー: " + err.message);
            });
    });
};
