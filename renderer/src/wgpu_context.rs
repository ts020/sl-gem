//! WGPUの初期化と管理を担当するモジュール

use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::window::Window;

/// WGPUコンテキスト
///
/// WGPUの初期化と管理を担当する構造体です。
/// デバイス、キュー、サーフェス、レンダリングパイプラインなどのWGPUリソースを管理します。
pub struct WgpuContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface,
    pub surface_config: SurfaceConfiguration,
    pub render_pipeline: Option<RenderPipeline>,
    pub window_size: winit::dpi::PhysicalSize<u32>,
}

impl WgpuContext {
    /// 新しいWGPUコンテキストを作成
    pub async fn new(window: &Window) -> Result<Self> {
        // ウィンドウサイズを取得
        let window_size = window.inner_size();

        // WGPUインスタンスを作成
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // サーフェスを作成
        let surface = unsafe { instance.create_surface(&window) }?;

        // アダプタを要求
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("適切なアダプタが見つかりませんでした"))?;

        // デバイスとキューを作成
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Primary Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // デバイスとキューをArcでラップ
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // サーフェスの設定
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline: None,
            window_size,
        })
    }

    /// ヘッドレスコンテキストを作成（オフスクリーンレンダリング用）
    pub async fn new_headless(width: u32, height: u32) -> Result<Self> {
        // WGPUインスタンスを作成
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // ヘッドレスモードでアダプタを要求
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("適切なアダプタが見つかりませんでした"))?;

        // デバイスとキューを作成
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Headless Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // デバイスとキューをArcでラップ
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // ダミーサーフェス設定
        let window_size = winit::dpi::PhysicalSize::new(width, height);

        // テクスチャフォーマット（通常はBgra8UnormSrgbを使用）
        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        // アダプタからサーフェス機能を取得（ヘッドレスモード用）
        let dummy_surface_caps = adapter.get_texture_format_features(surface_format);
        // wgpu 0.16ではAlphaModeを使用
        let alpha_mode = wgpu::CompositeAlphaMode::Auto;

        // ダミーサーフェス設定
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
        };

        // ヘッドレスモードではダミーのサーフェスを作成
        // 実際のレンダリングはテクスチャに対して行う
        let surface = unsafe {
            // ダミーウィンドウを作成して対応するサーフェスを取得
            let event_loop = winit::event_loop::EventLoop::new();
            let window = winit::window::WindowBuilder::new()
                .with_visible(false)
                .build(&event_loop)
                .unwrap();
            instance.create_surface(&window)?
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline: None,
            window_size,
        })
    }

    /// ウィンドウサイズが変更されたときに呼び出されるメソッド
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// フレームの描画準備
    pub fn prepare_frame(&self) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView)> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok((output, view))
    }

    /// コマンドバッファをGPUに送信
    pub fn submit_commands(&self, command_buffer: wgpu::CommandBuffer) {
        self.queue.submit(std::iter::once(command_buffer));
    }

    /// レンダリングパイプラインを設定
    pub fn set_render_pipeline(&mut self, pipeline: RenderPipeline) {
        self.render_pipeline = Some(pipeline);
    }

    /// 基本的なレンダリングパイプラインを作成
    pub fn create_basic_pipeline(
        &self,
        shader_source: &str,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Result<RenderPipeline> {
        // シェーダーモジュールを作成
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Basic Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        // パイプラインレイアウトを作成
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Basic Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        // レンダリングパイプラインを作成
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Basic Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_config.format,
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

        Ok(pipeline)
    }

    /// オフスクリーンレンダリング用のテクスチャを作成
    pub fn create_render_texture(&self) -> Result<(wgpu::Texture, wgpu::TextureView)> {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width: self.window_size.width,
                height: self.window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok((texture, view))
    }
}
