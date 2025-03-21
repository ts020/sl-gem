# SL-GEM Engine

ターン制ウォーSLGゲームのためのゲームエンジンを提供します。

## 主な機能

- エンジンループの管理
- イベントシステム（Pub/Sub方式）
- 戦略エンジン
  - マップ管理
  - 勢力システム
  - ターン管理
  - AI処理
- 戦闘エンジン
  - 戦場マップ
  - ユニット操作
  - 戦闘処理

## アーキテクチャ

```plaintext
src/
├── core/      # エンジン基盤
├── strategy/  # 戦略ゲーム機能
├── gui/       # GUI実装
└── main.rs    # エントリーポイント
```

## 技術スタック

- Rust
- Egui (GUI)
- WGPU (レンダリング)