//! シェーダーテストランナー
//!
//! シェーダーテストを実行するためのランナーを提供します。

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::{OutputValidator, TestCase, ValidationResult};
use crate::shader_test::ShaderSource;
use crate::texture::TextureGenerator;
use crate::{Texture, WgpuContext};

/// シェーダーテストランナー
///
/// シェーダーのテストケースを実行するためのモジュールです。
pub struct ShaderTestRunner {
    /// WGPUコンテキスト
    wgpu_context: WgpuContext,
    /// テストケース
    test_case: Option<TestCase>,
    /// テクスチャ
    texture: Option<Texture>,
    /// ユニフォームバッファ
    uniform_buffer: Option<wgpu::Buffer>,
    /// ユニフォームバインドグループ
    uniform_bind_group: Option<wgpu::BindGroup>,
    /// テクスチャバインドグループ
    texture_bind_group: Option<wgpu::BindGroup>,
    /// レンダーパイプライン
    render_pipeline: Option<wgpu::RenderPipeline>,
    /// 頂点バッファ
    vertex_buffer: Option<wgpu::Buffer>,
    /// インデックスバッファ
    index_buffer: Option<wgpu::Buffer>,
    /// 出力テクスチャ
    output_texture: Option<Texture>,
    /// 実行時間（秒）
    time: f32,
}

impl ShaderTestRunner {
    /// 新しいシェーダーテストランナーを作成
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        // ヘッドレスモードでWGPUコンテキストを初期化
        let wgpu_context = WgpuContext::new_headless(width, height).await?;

        Ok(Self {
            wgpu_context,
            test_case: None,
            texture: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            texture_bind_group: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            output_texture: None,
            time: 0.0,
        })
    }

    /// すでに存在するWGPUコンテキストからシェーダーテストランナーを作成
    pub fn new_with_context(wgpu_context: WgpuContext) -> Self {
        Self {
            wgpu_context,
            test_case: None,
            texture: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            texture_bind_group: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            output_texture: None,
            time: 0.0,
        }
    }

    /// テストケースを設定
    pub fn set_test_case(&mut self, test_case: TestCase) {
        self.test_case = Some(test_case);
        self.reset_resources();
    }

    /// 時間を設定
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    /// 時間を進める
    pub fn advance_time(&mut self, delta_time: f32) {
        self.time += delta_time;
    }

    /// テスト環境をリセット
    fn reset_resources(&mut self) {
        self.texture = None;
        self.uniform_buffer = None;
        self.uniform_bind_group = None;
        self.texture_bind_group = None;
        self.render_pipeline = None;
        self.vertex_buffer = None;
        self.index_buffer = None;
        self.output_texture = None;
    }

    /// テストケースのリソースを初期化
    pub fn initialize_resources(&mut self) -> Result<()> {
        let test_case = match &self.test_case {
            Some(tc) => tc,
            None => return Err(anyhow::anyhow!("テストケースが設定されていません")),
        };

        // 出力テクスチャを作成
        let (width, height) = test_case.output_size();
        let output_texture = Texture::new_render_target(
            &self.wgpu_context.device,
            width,
            height,
            Some("Test Output Texture"),
            self.wgpu_context.surface_config.format,
        );
        self.output_texture = Some(output_texture);

        // テクスチャを読み込む（または生成する）
        let texture = match test_case.texture_path() {
            Some(path) => Texture::from_file(
                &self.wgpu_context.device,
                &self.wgpu_context.queue,
                path,
                Some("Test Texture"),
            )
            .context("テクスチャの読み込みに失敗しました")?,
            None => TextureGenerator::test_pattern(
                &self.wgpu_context.device,
                &self.wgpu_context.queue,
                256,
                256,
            ),
        };
        self.texture = Some(texture);

        // 頂点バッファを作成
        let vertex_buffer =
            self.wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(test_case.vertex_data()),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        self.vertex_buffer = Some(vertex_buffer);

        // インデックスバッファを作成（存在する場合）
        let index_data_opt = test_case.index_data();
        if let Some(indices) = index_data_opt {
            let index_buffer =
                self.wgpu_context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
            self.index_buffer = Some(index_buffer);
        }

        // ユニフォームバッファを作成
        let uniform_data = test_case.create_uniform_buffer(self.time);
        let uniform_buffer =
            self.wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: &uniform_data,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        self.uniform_buffer = Some(uniform_buffer);

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

        // テクスチャバインドグループレイアウトを作成
        let texture_bind_group_layout =
            Texture::create_bind_group_layout(&self.wgpu_context.device);

        // ユニフォームバインドグループを作成
        let uniform_bind_group =
            self.wgpu_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Uniform Bind Group"),
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                    }],
                });
        self.uniform_bind_group = Some(uniform_bind_group);

        // テクスチャバインドグループを作成
        let texture = self.texture.as_ref().unwrap();
        let texture_bind_group =
            texture.create_bind_group(&self.wgpu_context.device, &texture_bind_group_layout);
        self.texture_bind_group = Some(texture_bind_group);

        // シェーダーを読み込み
        let shader_module = self.load_shader(test_case.shader())?;

        // レンダーパイプラインを作成
        let render_pipeline =
            self.wgpu_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Test Render Pipeline"),
                    layout: Some(&self.wgpu_context.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("Test Pipeline Layout"),
                            bind_group_layouts: &[
                                &uniform_bind_group_layout,
                                &texture_bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        },
                    )),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[super::super::Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.wgpu_context.surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });
        self.render_pipeline = Some(render_pipeline);

        Ok(())
    }

    /// シェーダーをロード
    fn load_shader(&self, shader_source: &ShaderSource) -> Result<wgpu::ShaderModule> {
        match shader_source {
            ShaderSource::BuiltIn(name) => {
                let source = match name.as_str() {
                    "test" => super::super::shaders::TEST_SHADER,
                    "tile" => super::super::shaders::TILE_SHADER,
                    "unit" => super::super::shaders::UNIT_SHADER,
                    "ui" => super::super::shaders::UI_SHADER,
                    _ => return Err(anyhow::anyhow!("未知の組み込みシェーダー: {}", name)),
                };
                Ok(self
                    .wgpu_context
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(&format!("{} Shader", name)),
                        source: wgpu::ShaderSource::Wgsl(source.into()),
                    }))
            }
            ShaderSource::Code(code) => {
                Ok(self
                    .wgpu_context
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Custom Shader"),
                        source: wgpu::ShaderSource::Wgsl(code.as_str().into()),
                    }))
            }
            ShaderSource::File(path) => {
                let code = std::fs::read_to_string(path)
                    .context(format!("シェーダーファイルの読み込みに失敗: {:?}", path))?;
                Ok(self
                    .wgpu_context
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(&format!("File Shader: {:?}", path)),
                        source: wgpu::ShaderSource::Wgsl(code.into()),
                    }))
            }
        }
    }

    /// ユニフォームバッファを更新
    pub fn update_uniforms(&self) -> Result<()> {
        let test_case = match &self.test_case {
            Some(tc) => tc,
            None => return Err(anyhow::anyhow!("テストケースが設定されていません")),
        };

        let uniform_buffer = match &self.uniform_buffer {
            Some(buffer) => buffer,
            None => {
                return Err(anyhow::anyhow!(
                    "ユニフォームバッファが初期化されていません"
                ))
            }
        };

        let uniform_data = test_case.create_uniform_buffer(self.time);
        self.wgpu_context
            .queue
            .write_buffer(uniform_buffer, 0, &uniform_data);

        Ok(())
    }

    /// テストケースを実行
    pub fn run(&mut self) -> Result<ValidationResult> {
        // テストケースのクローンを取得して所有権の問題を回避
        let test_case = match &self.test_case {
            Some(tc) => tc.clone(),
            None => return Err(anyhow::anyhow!("テストケースが設定されていません")),
        };

        // リソースが初期化されていなければ初期化
        if self.render_pipeline.is_none() {
            self.initialize_resources()?;
        }

        // ユニフォームを更新
        self.update_uniforms()?;

        // レンダリング
        let output_texture = self.output_texture.as_ref().unwrap();
        self.render_to_texture(&output_texture.view)?;

        // テクスチャデータを読み取り
        let output_data =
            output_texture.read_pixels(&self.wgpu_context.device, &self.wgpu_context.queue)?;

        // 検証関数があれば実行
        if let Some(ref validation_fn) = test_case.validation_function {
            return Ok(validation_fn(
                &output_data,
                test_case.output_size().0,
                test_case.output_size().1,
            ));
        }

        // 検証関数がなければ成功とみなす
        Ok(ValidationResult::success())
    }

    /// テクスチャにレンダリング
    fn render_to_texture(&self, texture_view: &wgpu::TextureView) -> Result<()> {
        let test_case = match &self.test_case {
            Some(tc) => tc,
            None => return Err(anyhow::anyhow!("テストケースが設定されていません")),
        };

        let render_pipeline = match &self.render_pipeline {
            Some(pipeline) => pipeline,
            None => {
                return Err(anyhow::anyhow!(
                    "レンダーパイプラインが初期化されていません"
                ))
            }
        };

        let uniform_bind_group = match &self.uniform_bind_group {
            Some(group) => group,
            None => {
                return Err(anyhow::anyhow!(
                    "ユニフォームバインドグループが初期化されていません"
                ))
            }
        };

        let texture_bind_group = match &self.texture_bind_group {
            Some(group) => group,
            None => {
                return Err(anyhow::anyhow!(
                    "テクスチャバインドグループが初期化されていません"
                ))
            }
        };

        let vertex_buffer = match &self.vertex_buffer {
            Some(buffer) => buffer,
            None => return Err(anyhow::anyhow!("頂点バッファが初期化されていません")),
        };

        // コマンドエンコーダを作成
        let mut encoder =
            self.wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Test Render Encoder"),
                });

        // レンダーパスを開始
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Test Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: test_case.background_color()[0] as f64,
                            g: test_case.background_color()[1] as f64,
                            b: test_case.background_color()[2] as f64,
                            a: test_case.background_color()[3] as f64,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, uniform_bind_group, &[]);
            render_pass.set_bind_group(1, texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            // インデックスバッファがあれば使用
            if let Some(ref index_buffer) = self.index_buffer {
                let index_data_opt = test_case.index_data();
                if let Some(indices) = index_data_opt {
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
            } else {
                // インデックスバッファがなければ通常の描画
                render_pass.draw(0..test_case.vertex_data().len() as u32, 0..1);
            }
        }

        // コマンドを実行
        self.wgpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// 出力画像を取得
    pub fn get_output_image(&self) -> Result<image::RgbaImage> {
        let test_case = match &self.test_case {
            Some(tc) => tc,
            None => return Err(anyhow::anyhow!("テストケースが設定されていません")),
        };

        let output_texture = match &self.output_texture {
            Some(texture) => texture,
            None => return Err(anyhow::anyhow!("出力テクスチャが初期化されていません")),
        };

        let output_data =
            output_texture.read_pixels(&self.wgpu_context.device, &self.wgpu_context.queue)?;

        let width = test_case.output_size().0;
        let height = test_case.output_size().1;

        // RgbaImageに変換
        let image = image::RgbaImage::from_raw(width, height, output_data)
            .ok_or_else(|| anyhow::anyhow!("出力データから画像を作成できません"))?;

        Ok(image)
    }

    /// 出力テクスチャを取得
    pub fn get_output_texture_view(&self) -> Option<&wgpu::TextureView> {
        self.output_texture.as_ref().map(|tex| &tex.view)
    }

    /// 出力をファイルに保存
    pub fn save_output_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let image = self.get_output_image()?;
        image.save(path)?;
        Ok(())
    }

    /// WGPUデバイスを取得
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.wgpu_context.device
    }

    /// WGPUキューを取得
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.wgpu_context.queue
    }
}
