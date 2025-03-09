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

    /// レンダリングターゲット用のテクスチャを作成
    pub fn new_render_target(
        device: &Arc<Device>,
        width: u32,
        height: u32,
        label: Option<&str>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // レンダリングターゲット用のテクスチャを作成
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // テクスチャビューを作成
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // サンプラーを作成
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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

    /// テクスチャのピクセルデータを取得
    pub fn read_pixels(&self, device: &Arc<Device>, queue: &Arc<Queue>) -> Result<Vec<u8>> {
        // バッファサイズを計算
        let buffer_size = (4 * self.size.0 * self.size.1) as wgpu::BufferAddress;

        // バッファを作成
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Read Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // コマンドエンコーダーを作成
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Read Encoder"),
        });

        // テクスチャからバッファにコピー
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.0),
                    rows_per_image: Some(self.size.1),
                },
            },
            wgpu::Extent3d {
                width: self.size.0,
                height: self.size.1,
                depth_or_array_layers: 1,
            },
        );

        // コマンドを実行
        queue.submit(std::iter::once(encoder.finish()));

        // バッファをマップしてデータを取得
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);

        rx.recv().unwrap()?;

        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();

        drop(data);
        output_buffer.unmap();

        Ok(result)
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
    pub fn new(texture: Texture, tile_width: u32, tile_height: u32) -> Self {
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

/// テクスチャジェネレーター
///
/// シェーダーテスト用のテクスチャを生成するユーティリティ
pub struct TextureGenerator;

impl TextureGenerator {
    /// チェッカーボードパターンのテクスチャを生成
    pub fn checker_pattern(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        width: u32,
        height: u32,
        cell_size: u32,
        color1: [u8; 4],
        color2: [u8; 4],
    ) -> Texture {
        let mut data = vec![0u8; (width * height * 4) as usize];

        for y in 0..height {
            for x in 0..width {
                let cell_x = x / cell_size;
                let cell_y = y / cell_size;
                let is_color1 = (cell_x + cell_y) % 2 == 0;
                let color = if is_color1 { color1 } else { color2 };

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = color[3];
            }
        }

        Texture::new(
            device,
            queue,
            width,
            height,
            Some("Checker Pattern Texture"),
            Some(&data),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    /// グラデーションのテクスチャを生成
    pub fn gradient(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        width: u32,
        height: u32,
        start_color: [u8; 4],
        end_color: [u8; 4],
        horizontal: bool,
    ) -> Texture {
        let mut data = vec![0u8; (width * height * 4) as usize];

        for y in 0..height {
            for x in 0..width {
                let progress = if horizontal {
                    x as f32 / (width as f32 - 1.0)
                } else {
                    y as f32 / (height as f32 - 1.0)
                };

                let r = (start_color[0] as f32 * (1.0 - progress) + end_color[0] as f32 * progress)
                    as u8;
                let g = (start_color[1] as f32 * (1.0 - progress) + end_color[1] as f32 * progress)
                    as u8;
                let b = (start_color[2] as f32 * (1.0 - progress) + end_color[2] as f32 * progress)
                    as u8;
                let a = (start_color[3] as f32 * (1.0 - progress) + end_color[3] as f32 * progress)
                    as u8;

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = a;
            }
        }

        Texture::new(
            device,
            queue,
            width,
            height,
            Some("Gradient Texture"),
            Some(&data),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    /// 単色のテクスチャを生成
    pub fn solid_color(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Texture {
        let data = vec![color[0], color[1], color[2], color[3]].repeat((width * height) as usize);

        Texture::new(
            device,
            queue,
            width,
            height,
            Some("Solid Color Texture"),
            Some(&data),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    /// テストパターンのテクスチャを生成
    pub fn test_pattern(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        width: u32,
        height: u32,
    ) -> Texture {
        let mut data = vec![0u8; (width * height * 4) as usize];

        // 領域を4分割して異なるパターンを描画
        let half_width = width / 2;
        let half_height = height / 2;

        // 左上: 赤から緑へのグラデーション
        for y in 0..half_height {
            for x in 0..half_width {
                let r = 255 - (255 * x / half_width) as u8;
                let g = (255 * x / half_width) as u8;

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            }
        }

        // 右上: チェッカーボード
        for y in 0..half_height {
            for x in half_width..width {
                let cell_x = (x - half_width) / 16;
                let cell_y = y / 16;
                let is_white = (cell_x + cell_y) % 2 == 0;
                let color = if is_white { 255 } else { 0 };

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = color;
                data[idx + 1] = color;
                data[idx + 2] = color;
                data[idx + 3] = 255;
            }
        }

        // 左下: 同心円
        let center_x = half_width / 2;
        let center_y = half_height + half_height / 2;
        let max_radius = half_width.min(half_height) as f32;

        for y in half_height..height {
            for x in 0..half_width {
                let dx = x as f32 - center_x as f32;
                let dy = y as f32 - center_y as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                let normalized_distance = distance / max_radius;
                let ring = (normalized_distance * 10.0) as u8 % 2;

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = if ring == 0 { 255 } else { 0 };
                data[idx + 1] = 0;
                data[idx + 2] = if ring == 0 { 0 } else { 255 };
                data[idx + 3] = 255;
            }
        }

        // 右下: HSV色空間
        for y in half_height..height {
            for x in half_width..width {
                let h = (x - half_width) as f32 / half_width as f32;
                let s = 1.0;
                let v = (y - half_height) as f32 / half_height as f32;

                // HSV to RGB変換
                let c = v * s;
                let x_val = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
                let m = v - c;

                let (r, g, b) = if h < 1.0 / 6.0 {
                    (c, x_val, 0.0)
                } else if h < 2.0 / 6.0 {
                    (x_val, c, 0.0)
                } else if h < 3.0 / 6.0 {
                    (0.0, c, x_val)
                } else if h < 4.0 / 6.0 {
                    (0.0, x_val, c)
                } else if h < 5.0 / 6.0 {
                    (x_val, 0.0, c)
                } else {
                    (c, 0.0, x_val)
                };

                let idx = ((y * width + x) * 4) as usize;
                data[idx] = ((r + m) * 255.0) as u8;
                data[idx + 1] = ((g + m) * 255.0) as u8;
                data[idx + 2] = ((b + m) * 255.0) as u8;
                data[idx + 3] = 255;
            }
        }

        Texture::new(
            device,
            queue,
            width,
            height,
            Some("Test Pattern Texture"),
            Some(&data),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }
}
