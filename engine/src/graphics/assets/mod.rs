//! アセット管理モジュール
//!
//! テクスチャやその他のゲームアセットを管理します。

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use wgpu::{Device, Queue};

use crate::graphics::texture::{Texture, TextureAtlas};

/// テクスチャID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureId {
    TileSet,
    UnitSet,
    Effects,
    UI,
    Custom(u32),
}

/// アトラスID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtlasId {
    Tiles,
    Units,
    Effects,
    Custom(u32),
}

/// アセットマネージャー
///
/// テクスチャやその他のゲームアセットを管理します。
pub struct AssetManager {
    textures: HashMap<TextureId, Texture>,
    texture_atlases: HashMap<AtlasId, TextureAtlas>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl AssetManager {
    /// 新しいアセットマネージャーを作成
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            textures: HashMap::new(),
            texture_atlases: HashMap::new(),
            device,
            queue,
        }
    }

    /// テクスチャを読み込む
    pub fn load_texture<P: AsRef<Path>>(
        &mut self,
        id: TextureId,
        path: P,
        label: Option<&str>,
    ) -> Result<()> {
        let texture = Texture::from_file(&self.device, &self.queue, path, label)?;
        self.textures.insert(id, texture);
        Ok(())
    }

    /// テクスチャアトラスを作成
    pub fn create_atlas(
        &mut self,
        id: AtlasId,
        texture_id: TextureId,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<()> {
        let texture = self
            .textures
            .get(&texture_id)
            .ok_or_else(|| anyhow::anyhow!("テクスチャが見つかりません: {:?}", texture_id))?;

        let atlas = TextureAtlas::new(
            Texture::new(
                &self.device,
                &self.queue,
                texture.size.0,
                texture.size.1,
                None,
                None,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
            tile_width,
            tile_height,
        );

        self.texture_atlases.insert(id, atlas);
        Ok(())
    }

    /// テクスチャアトラスをファイルから直接読み込む
    pub fn load_atlas<P: AsRef<Path>>(
        &mut self,
        id: AtlasId,
        path: P,
        tile_width: u32,
        tile_height: u32,
        label: Option<&str>,
    ) -> Result<()> {
        let atlas = TextureAtlas::from_file(
            &self.device,
            &self.queue,
            path,
            tile_width,
            tile_height,
            label,
        )?;
        self.texture_atlases.insert(id, atlas);
        Ok(())
    }

    /// テクスチャを取得
    pub fn get_texture(&self, id: TextureId) -> Option<&Texture> {
        self.textures.get(&id)
    }

    /// テクスチャアトラスを取得
    pub fn get_atlas(&self, id: AtlasId) -> Option<&TextureAtlas> {
        self.texture_atlases.get(&id)
    }

    /// デバイスへの参照を取得
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// キューへの参照を取得
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// デフォルトのタイルセットを読み込む
    pub fn load_default_tileset<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // タイルセットテクスチャを読み込む
        self.load_texture(TextureId::TileSet, path, Some("Default Tileset"))?;

        // タイルアトラスを作成（32x32ピクセルのタイル）
        self.create_atlas(AtlasId::Tiles, TextureId::TileSet, 32, 32)?;

        Ok(())
    }

    /// デフォルトのユニットセットを読み込む
    pub fn load_default_unitset<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // ユニットセットテクスチャを読み込む
        self.load_texture(TextureId::UnitSet, path, Some("Default Unitset"))?;

        // ユニットアトラスを作成（32x32ピクセルのスプライト）
        self.create_atlas(AtlasId::Units, TextureId::UnitSet, 32, 32)?;

        Ok(())
    }
}

/// アセットの初期化
///
/// デフォルトのアセットを読み込みます。
pub async fn initialize_assets(device: Arc<Device>, queue: Arc<Queue>) -> Result<AssetManager> {
    let mut asset_manager = AssetManager::new(device, queue);

    // デフォルトのアセットを読み込む
    // game/assetsディレクトリからアセットを読み込む
    let tileset_path = "game/assets/textures/tiles/default_tileset.png";
    let unitset_path = "game/assets/textures/units/default_unitset.png";

    // 注: 実際の実装では、アセットのパスは設定ファイルから読み込むか、
    // 環境に応じて適切に設定する必要があります。

    // アセットが存在するか確認し、存在する場合は読み込む
    if std::path::Path::new(tileset_path).exists() {
        match asset_manager.load_default_tileset(tileset_path) {
            Ok(_) => println!("タイルセットを読み込みました: {}", tileset_path),
            Err(e) => println!("タイルセットの読み込みに失敗しました: {}", e),
        }
    } else {
        println!(
            "警告: タイルセットファイルが見つかりません: {}",
            tileset_path
        );
        println!("デフォルトのタイルセットを使用します。");
        // 実際のアセットが用意されるまでは、ダミーのテクスチャを使用
    }

    if std::path::Path::new(unitset_path).exists() {
        match asset_manager.load_default_unitset(unitset_path) {
            Ok(_) => println!("ユニットセットを読み込みました: {}", unitset_path),
            Err(e) => println!("ユニットセットの読み込みに失敗しました: {}", e),
        }
    } else {
        println!(
            "警告: ユニットセットファイルが見つかりません: {}",
            unitset_path
        );
        println!("デフォルトのユニットセットを使用します。");
        // 実際のアセットが用意されるまでは、ダミーのテクスチャを使用
    }

    Ok(asset_manager)
}
