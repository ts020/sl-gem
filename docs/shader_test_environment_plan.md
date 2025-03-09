# シェーダーテスト環境計画

## 概要

シェーダー開発と検証のための専用環境を実装します。この環境は以下の機能を提供します：

1. インタラクティブなシェーダー編集と実行
2. パラメータ調整機能
3. 自動テスト検証
4. CI/CD環境での実行サポート

## アーキテクチャ

```
renderer/
├── src/
│   ├── lib.rs              # クレートルート
│   ├── camera.rs           # カメラ機能
│   ├── texture.rs          # テクスチャ管理
│   ├── wgpu_context.rs     # WGPU基盤
│   ├── window.rs           # ウィンドウ管理
│   ├── shaders/            # シェーダー定義
│   │   ├── mod.rs
│   │   ├── tile.wgsl       # タイルシェーダー
│   │   ├── unit.wgsl       # ユニットシェーダー
│   │   ├── ui.wgsl         # UIシェーダー
│   │   └── test.wgsl       # テスト用シェーダー
│   └── shader_test/        # テスト環境
│       ├── mod.rs          # モジュール定義
│       ├── case.rs         # テストケース定義
│       ├── validator.rs    # 出力検証器
│       ├── runner.rs       # シェーダー実行
│       ├── ui.rs           # インタラクティブUI
│       └── headless.rs     # ヘッドレス実行
└── examples/
    └── shader_tester.rs    # テスト環境のエントリポイント
```

## 実装状況

現在、基本的なモジュール構造は実装済みですが、依存関係の競合により完全なコンパイルはできていません。以下の問題が残っています：

1. wgpuのバージョン競合
2. egui関連APIの不一致
3. インターフェースの整合性

## 今後の作業

1. バージョン競合の解決
   - プロジェクト全体で一貫したwgpuバージョンを使用
   - egui依存関係の同期

2. 最小限の動作バージョンの作成
   - 基本的なシェーダーロードとレンダリング
   - シンプルなテストケース実行

3. 段階的な機能追加
   - インタラクティブなUIコンポーネント
   - テスト検証システムの実装
   - CI統合

## 使用方法（将来計画）

```bash
# インタラクティブモード
cargo run --package renderer --example shader_tester

# ヘッドレスモード
cargo run --package renderer --example shader_tester -- --headless --report report.html