# Milestone

- [ ] Initial project setup

## Completed

- Added rust-analyzer configuration to .rust-analyzer/config:
  - rust-analyzer.inlayHints.enable: true
  - rust-analyzer.cargo.loadOutDirsFromCheck: true
  - rust-analyzer.files.excludeDirs: ["target"]
  - rust-analyzer.procMacro.enable: false
  - rust-analyzer.checkOnSave: false
  - rust-analyzer.check.command: "clippy"

- [x] Basic Event System Implementation (2025/03/08)
  - Pub/Subパターンによるイベントシステムの実装
  - イベントバスを用いたコンポーネント間通信
  - ゲームループの基本実装
    - イベントキューの処理
    - フレーム制御
    - 終了処理
  - [x] イベントシステムの型安全性向上 (2025/03/08)
    - `PrioritizedEvent`と`GameEvent`の型互換性の問題を解決
    - イベント変換レイヤーの導入によりAPI一貫性を確保
