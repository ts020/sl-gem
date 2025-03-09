//! マップレンダラー
//!
//! マップとユニットのレンダリングを担当します。

use anyhow::Result;
use glam::Mat4;
use std::collections::HashMap;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::graphics::{
    assets::AssetManager,
    camera::Camera,
    renderer::{
        tile_renderer::TileRenderer, ui_renderer::UIRenderer, unit_renderer::UnitRenderer, Uniforms,
    },
    wgpu_context::WgpuContext,
};
use crate::gui::map_gui::MapViewOptions;
use model::{Map, Unit};

/// マップレンダラー
pub struct MapRenderer {
    wgpu_context: WgpuContext,
    camera: Camera,
    asset_manager: AssetManager,
    tile_renderer: Option<TileRenderer>,
    unit_renderer: Option<UnitRenderer>,
    ui_renderer: Option<UIRenderer>,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    start_time: Instant,
}

impl MapRenderer {
    /// 新しいマップレンダラーを作成
    pub async fn new(window: &Window) -> Result<Self> {
        // WGPUコンテキストを初期化
        let wgpu_context = WgpuContext::new(window).await?;

        // カメラを初期化
        let size = window.inner_size();
        let camera = Camera::new(size.width as f32, size.height as f32);

        // アセットマネージャーを初期化
        let asset_manager =
            AssetManager::new(wgpu_context.device.clone(), wgpu_context.queue.clone());

        // ユニフォームバッファを作成
        let uniforms = Uniforms {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            time: 0.0,
            _padding: [0.0; 3],
        };

        let uniform_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        // ユニフォームバインドグループレイアウトを作成
        let uniform_bind_group_layout =
            wgpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // ユニフォームバインドグループを作成
        let uniform_bind_group =
            wgpu_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Uniform Bind Group"),
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                });

        // レンダラーは後で初期化する

        Ok(Self {
            wgpu_context,
            camera,
            asset_manager,
            tile_renderer: None,
            unit_renderer: None,
            ui_renderer: None,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            start_time: Instant::now(),
        })
    }

    /// レンダラーを初期化
    fn initialize_renderers(&mut self) -> Result<()> {
        // ユニフォームバインドグループレイアウトを作成
        let uniform_bind_group_layout =
            self.wgpu_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // レンダラーを初期化
        self.tile_renderer = Some(TileRenderer::new(
            &self.wgpu_context,
            &uniform_bind_group_layout,
        )?);
        self.unit_renderer = Some(UnitRenderer::new(
            &self.wgpu_context,
            &uniform_bind_group_layout,
        )?);
        self.ui_renderer = Some(UIRenderer::new(
            &self.wgpu_context,
            &uniform_bind_group_layout,
        )?);

        Ok(())
    }

    /// アセットを読み込む
    pub fn load_assets<P: AsRef<std::path::Path>>(
        &mut self,
        tileset_path: P,
        unitset_path: P,
    ) -> Result<()> {
        // タイルセットを読み込む
        self.asset_manager.load_default_tileset(tileset_path)?;

        // ユニットセットを読み込む
        self.asset_manager.load_default_unitset(unitset_path)?;

        Ok(())
    }

    /// ビューポートを更新
    pub fn update_viewport(&mut self, width: u32, height: u32) {
        self.wgpu_context
            .resize(winit::dpi::PhysicalSize::new(width, height));
        self.camera.update_viewport(width as f32, height as f32);
    }

    /// MapGUIのビュー設定からカメラを更新
    pub fn update_from_map_view_options(&mut self, options: &MapViewOptions) {
        // スクロール位置を設定
        self.camera
            .set_from_map_gui_scroll(options.scroll_x, options.scroll_y, options.tile_size);

        // ズームを設定
        self.camera.set_from_map_gui_zoom(options.zoom);
    }

    /// マップとユニットをレンダリング
    pub fn render(
        &mut self,
        map: &Map,
        units: &HashMap<u32, Unit>,
        options: &MapViewOptions,
    ) -> Result<()> {
        // レンダラーが初期化されていない場合は初期化
        if self.tile_renderer.is_none() {
            self.initialize_renderers()?;
        }

        // MapGUIのビュー設定からカメラを更新
        self.update_from_map_view_options(options);

        // 経過時間を計算
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // ユニフォームバッファを更新
        self.uniforms.view_proj = self.camera.view_projection_matrix().to_cols_array_2d();
        self.uniforms.time = elapsed;
        self.wgpu_context.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // フレームの描画準備
        let (output, view) = self.wgpu_context.prepare_frame()?;

        // コマンドエンコーダを作成
        let mut encoder =
            self.wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // テクスチャバインドグループレイアウトを作成
        let texture_bind_group_layout =
            crate::graphics::texture::Texture::create_bind_group_layout(&self.wgpu_context.device);

        // タイルテクスチャバインドグループを作成
        let tile_texture;
        let tile_texture_bind_group;

        if let Some(tileset_texture) = self
            .asset_manager
            .get_texture(crate::graphics::assets::TextureId::TileSet)
        {
            // タイルセットテクスチャが読み込まれている場合はそれを使用
            tile_texture_bind_group = tileset_texture
                .create_bind_group(&self.wgpu_context.device, &texture_bind_group_layout);
            println!("タイルセットテクスチャを使用します");
        } else {
            // テクスチャが読み込まれていない場合はダミーテクスチャを使用
            // 純白のダミーテクスチャを使用（色乗算に影響しない）
            tile_texture = crate::graphics::texture::Texture::new(
                &self.wgpu_context.device,
                &self.wgpu_context.queue,
                1,
                1,                                   // 1x1ピクセルのデフォルトテクスチャ
                Some("Dummy Tile Texture"),          // ラベル
                Some(&[255u8, 255u8, 255u8, 255u8]), // 白色（u8型を明示）
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );
            tile_texture_bind_group = tile_texture
                .create_bind_group(&self.wgpu_context.device, &texture_bind_group_layout);
            println!("警告: タイルセットテクスチャが読み込まれていないため、純白のダミーテクスチャを使用します");
            // デバッグ: ダミーテクスチャのサイズと形式を出力
            println!("ダミータイルテクスチャ: サイズ=1x1, 形式=Rgba8UnormSrgb");
        }

        // ユニットテクスチャバインドグループを作成
        let unit_texture;
        let unit_texture_bind_group;

        if let Some(unitset_texture) = self
            .asset_manager
            .get_texture(crate::graphics::assets::TextureId::UnitSet)
        {
            // ユニットセットテクスチャが読み込まれている場合はそれを使用
            unit_texture_bind_group = unitset_texture
                .create_bind_group(&self.wgpu_context.device, &texture_bind_group_layout);
            println!("ユニットセットテクスチャを使用します");
        } else {
            // ユニットのテクスチャが見つからない場合は、ダミーテクスチャを使用
            unit_texture = crate::graphics::texture::Texture::new(
                &self.wgpu_context.device,
                &self.wgpu_context.queue,
                1,
                1,                                   // 1x1ピクセルのデフォルトテクスチャ
                Some("Dummy Unit Texture"),          // ラベル
                Some(&[255u8, 255u8, 255u8, 255u8]), // 白色（u8型を明示）
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );
            unit_texture_bind_group = unit_texture
                .create_bind_group(&self.wgpu_context.device, &texture_bind_group_layout);
            println!("警告: ユニットセットテクスチャが読み込まれていないため、純白のダミーテクスチャを使用します");
            println!("ダミーユニットテクスチャ: サイズ=1x1, 形式=Rgba8UnormSrgb");
        }

        // レンダーパスを作成
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // タイルをレンダリング
            if let Some(tile_renderer) = &mut self.tile_renderer {
                // ユニフォームバインドグループとテクスチャバインドグループを設定
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_bind_group(1, &tile_texture_bind_group, &[]);

                tile_renderer.render(
                    &mut render_pass,
                    map,
                    &self.uniform_bind_group,
                    options,
                    &self.wgpu_context.queue,
                );
            }

            // ユニットをレンダリング
            if let Some(unit_renderer) = &mut self.unit_renderer {
                // ユニフォームバインドグループとテクスチャバインドグループを設定
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_bind_group(1, &unit_texture_bind_group, &[]);

                unit_renderer.render(
                    &mut render_pass,
                    units,
                    &self.uniform_bind_group,
                    options,
                    &self.wgpu_context.queue,
                );
            }

            // UIをレンダリング
            if let Some(ui_renderer) = &mut self.ui_renderer {
                // ユニフォームバインドグループとテクスチャバインドグループを設定
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_bind_group(1, &tile_texture_bind_group, &[]);

                ui_renderer.render(
                    &mut render_pass,
                    &self.uniform_bind_group,
                    &self.wgpu_context.queue,
                );
            }
        }

        // コマンドバッファを送信
        self.wgpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // フレームを表示
        output.present();

        Ok(())
    }

    /// 入力イベントを処理
    pub fn handle_input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::Resized(new_size) => {
                self.update_viewport(new_size.width, new_size.height);
                true
            }
            _ => false,
        }
    }
}
