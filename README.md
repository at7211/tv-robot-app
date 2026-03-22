# TV-robot App

macOS menu bar 版本的 [TV-robot](https://github.com/at7211/TV-robot)，不需要開 terminal 就能使用。

## Features

- 選單列圖示 🎮，點開顯示 QR code
- 手機掃描 QR code 即可遙控電腦
- 遙控模式：播放/暫停、快轉/倒轉、音量調整
- 串流快捷鍵：全螢幕、靜音、略過片頭、字幕
- 滑鼠模式：觸控板、左右鍵、捲動

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run
```

或指定 port：

```sh
PORT=3001 cargo run
```

## Package as .app

```sh
cargo install cargo-bundle
cargo bundle --release
```

產生的 `.app` 在 `target/release/bundle/osx/` 目錄下，可以拖到 Applications 資料夾。

## Note

首次使用時 macOS 會要求授予「輔助使用」權限（System Settings > Privacy & Security > Accessibility），這是模擬鍵盤和滑鼠操作所需要的。
