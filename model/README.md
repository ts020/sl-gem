# SL-GEM Model

SL-GEMのゲームデータモデルを提供する共通ライブラリです。エンジンとデータエディタの両方で使用されます。

## 主な機能

- ゲームデータの構造定義
  - マップデータ
  - ユニットデータ
  - 勢力データ
  - 戦闘データ
- シリアライズ/デシリアライズ
- データ検証
- モデル間の関係性管理

## 技術スタック

- Rust
- Serde (シリアライズ/デシリアライズ)
- RON (Rusty Object Notation)

## 使用例

```rust
use sl_gem_model::{Map, Unit, Faction};

// マップデータの作成
let map = Map::new(/* params */);

// ユニットの配置
let unit = Unit::new(/* params */);
map.place_unit(unit, position);