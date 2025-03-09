//! UIレンダラー
//!
//! UI要素のレンダリングを担当します。

use anyhow::Result;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::graphics::{
    renderer::{UIInstance, Vertex},
    shaders::UI_SHADER,
};

/// UI要素のタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIElementType {
    /// テクスチャ
    Texture = 0,
    /// 単色
    SolidColor = 1,
    /// グラデーション
    Gradient = 2,
    /// 枠線付き四角形
    BorderedRect = 3,
}

/// UI要素
pub struct UIElement {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: Option<([f32; 2], [f32; 2])>,
    pub element_type: UIElementType,
}

/// UIレンダラー
pub struct UIRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    indices_len: u32,
    instances: Vec<UIInstance>,
    elements: Vec<UIElement>,
    max_instances: usize,
}

impl UIRenderer {
    /// 新しいUIレンダラーを作成
    pub fn new(
        wgpu_context: &crate::graphics::wgpu_context::WgpuContext,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        // テクスチャバインドグループレイアウトを作成
        let texture_bind_group_layout =
            crate::graphics::texture::Texture::create_bind_group_layout(&wgpu_context.device);

        // レンダリングパイプラインを作成
        let render_pipeline = wgpu_context.create_basic_pipeline(
            UI_SHADER,
            &[Vertex::desc(), UIInstance::desc()],
            &[uniform_bind_group_layout, &texture_bind_group_layout],
        )?;

        // 頂点バッファを作成（単一の四角形）
        let vertices = [
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
            }, // 左下
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            }, // 右下
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
            }, // 右上
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            }, // 左上
        ];

        let vertex_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("UI Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // インデックスバッファを作成（2つの三角形で四角形を形成）
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("UI Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // インスタンスバッファを作成（初期容量）
        let max_instances = 100; // 十分な数のUI要素をサポート
        let instance_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Instance Buffer"),
            size: (std::mem::size_of::<UIInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            indices_len: indices.len() as u32,
            instances: Vec::with_capacity(max_instances),
            elements: Vec::new(),
            max_instances,
        })
    }

    /// UI要素を追加
    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
    }

    /// UI要素をクリア
    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }

    /// UI要素からインスタンスデータを更新
    fn update_instances(&mut self) {
        self.instances.clear();

        for element in &self.elements {
            // テクスチャ座標を設定
            let (tex_coords_min, tex_coords_max) =
                element.tex_coords.unwrap_or(([0.0, 0.0], [1.0, 1.0]));

            // 位置と大きさを設定
            let position = Vec3::new(element.position[0], element.position[1], 0.2); // Z値を少し上げてタイルとユニットの上に表示

            // モデル行列を計算
            let model_matrix = Mat4::from_scale_rotation_translation(
                Vec3::new(element.size[0], element.size[1], 1.0),
                glam::Quat::IDENTITY,
                position,
            );

            // インスタンスを追加
            self.instances.push(UIInstance {
                model_matrix: model_matrix.to_cols_array_2d(),
                tex_coords_min,
                tex_coords_max,
                color: element.color,
                ui_type: element.element_type as u32,
                _padding: [0, 0, 0],
            });
        }
    }

    /// UI要素をレンダリング
    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _uniform_bind_group: &'a wgpu::BindGroup,
        queue: &wgpu::Queue,
    ) {
        // UI要素からインスタンスデータを更新
        self.update_instances();

        // インスタンスがない場合は何もしない
        if self.instances.is_empty() {
            return;
        }

        // インスタンスバッファを更新
        // これが重要！インスタンスデータをGPUに送信する
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        render_pass.set_pipeline(&self.render_pipeline);

        // バインドグループは既に設定されているはずなので、ここでは設定しない
        // render_pass.set_bind_group(0, uniform_bind_group, &[]);
        // render_pass.set_bind_group(1, texture_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices_len, 0, 0..self.instances.len() as u32);
    }

    /// ミニマップを追加
    pub fn add_minimap(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.add_element(UIElement {
            position: [x, y],
            size: [width, height],
            color: [1.0, 1.0, 1.0, 0.8],
            tex_coords: None,
            element_type: UIElementType::BorderedRect,
        });
    }

    /// 情報パネルを追加
    pub fn add_info_panel(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.add_element(UIElement {
            position: [x, y],
            size: [width, height],
            color: [0.2, 0.2, 0.2, 0.7],
            tex_coords: None,
            element_type: UIElementType::Gradient,
        });
    }

    /// ボタンを追加
    pub fn add_button(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.add_element(UIElement {
            position: [x, y],
            size: [width, height],
            color,
            tex_coords: None,
            element_type: UIElementType::BorderedRect,
        });
    }
}
