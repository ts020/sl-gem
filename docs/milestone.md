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

- [x] CIとコードベース改善 (2025/03/08)
  - GitHub Actionsとローカルチェックスクリプトの互換性確保
    - 依存関係の解析機能を統一
    - パッケージ影響範囲の検出精度を向上
  - コードの重複排除と構造改善
    - `LoopConfig`と`GameLoop`の実装統一
    - モジュール間の責任分担を最適化
  - 型の表示と使用方法の改善
    - `LogLevel`に`Display`トレイト実装
    - 最新のRust機能（`#[default]`属性）活用
