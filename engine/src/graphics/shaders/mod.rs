//! シェーダーモジュール
//!
//! WGSLシェーダーの管理を担当します。

/// タイルシェーダー
pub const TILE_SHADER: &str = include_str!("tile.wgsl");

/// ユニットシェーダー
pub const UNIT_SHADER: &str = include_str!("unit.wgsl");

/// UIシェーダー
pub const UI_SHADER: &str = include_str!("ui.wgsl");

#[cfg(test)]
mod tile_shader_test;
