//! WGPUの初期化と管理を担当するモジュール

use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, RenderPipeline};
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
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // パイプラインレイアウトを作成
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Basic Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        // レンダリングパイプラインを作成
        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
}