//! テクスチャ管理モジュール
//! 
//! テクスチャの読み込みと管理を担当します。

use anyhow::Result;
use image::GenericImageView;
use std::path::Path;
use std::sync::Arc;
use wgpu::{Device, Queue, Sampler, TextureView};

/// テクスチャ
/// 
/// WGPUテクスチャとそのビュー、サンプラーを管理します。
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub size: (u32, u32),
}

impl Texture {
    /// 新しいテクスチャを作成
    pub fn new(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        width: u32,
        height: u32,
        label: Option<&str>,
        data: Option<&[u8]>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // テクスチャを作成
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // データがある場合はテクスチャにコピー
        if let Some(data) = data {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );
        }

        // テクスチャビューを作成
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // サンプラーを作成
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // ピクセルアートにはNearestが適切
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size: (width, height),
        }
    }

    /// 画像ファイルからテクスチャを読み込む
    pub fn from_file<P: AsRef<Path>>(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        path: P,
        label: Option<&str>,
    ) -> Result<Self> {
        // 画像ファイルを読み込む
        let img = image::open(path)?;
        let dimensions = img.dimensions();
        
        // RGBAに変換
        let rgba = img.to_rgba8();
        let data = rgba.as_raw();

        // テクスチャを作成
        Ok(Self::new(
            device,
            queue,
            dimensions.0,
            dimensions.1,
            label,
            Some(data),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ))
    }

    /// バインドグループを作成
    pub fn create_bind_group(
        &self,
        device: &Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// テクスチャバインドグループレイアウトを作成
    pub fn create_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}

/// テクスチャアトラス
/// 
/// 複数のタイルを1つのテクスチャにまとめたアトラスを管理します。
pub struct TextureAtlas {
    pub texture: Texture,
    pub tile_size: (u32, u32),
    pub columns: u32,
    pub rows: u32,
}

impl TextureAtlas {
    /// 新しいテクスチャアトラスを作成
    pub fn new(
        texture: Texture,
        tile_width: u32,
        tile_height: u32,
    ) -> Self {
        let (texture_width, texture_height) = texture.size;
        let columns = texture_width / tile_width;
        let rows = texture_height / tile_height;

        Self {
            texture,
            tile_size: (tile_width, tile_height),
            columns,
            rows,
        }
    }

    /// 画像ファイルからテクスチャアトラスを読み込む
    pub fn from_file<P: AsRef<Path>>(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        path: P,
        tile_width: u32,
        tile_height: u32,
        label: Option<&str>,
    ) -> Result<Self> {
        let texture = Texture::from_file(device, queue, path, label)?;
        Ok(Self::new(texture, tile_width, tile_height))
    }

    /// タイルインデックスからUV座標を計算
    pub fn get_tile_uv(&self, index: u32) -> (f32, f32, f32, f32) {
        let col = index % self.columns;
        let row = index / self.columns;

        let u_min = col as f32 / self.columns as f32;
        let v_min = row as f32 / self.rows as f32;
        let u_max = (col + 1) as f32 / self.columns as f32;
        let v_max = (row + 1) as f32 / self.rows as f32;

        (u_min, v_min, u_max, v_max)
    }

    /// タイルタイプからUV座標を計算
    pub fn get_tile_uv_for_type(&self, tile_type: &model::CellType) -> (f32, f32, f32, f32) {
        let index = match tile_type {
            model::CellType::Plain => 0,
            model::CellType::Forest => 1,
            model::CellType::Mountain => 2,
            model::CellType::Water => 3,
            model::CellType::Road => 4,
            model::CellType::City => 5,
            model::CellType::Base => 6,
        };

        self.get_tile_uv(index)
    }
}