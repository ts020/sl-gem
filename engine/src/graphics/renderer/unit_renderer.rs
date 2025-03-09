//! ユニットレンダラー
//!
//! ユニットのレンダリングを担当します。

use anyhow::Result;
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::graphics::{
    renderer::{UnitInstance, Vertex},
    shaders::UNIT_SHADER,
};
use crate::gui::map_gui::MapViewOptions;
use model::{Unit, UnitType};

/// ユニットレンダラー
pub struct UnitRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    indices_len: u32,
    instances: Vec<UnitInstance>,
    max_instances: usize,
}

impl UnitRenderer {
    /// 新しいユニットレンダラーを作成
    pub fn new(
        wgpu_context: &crate::graphics::wgpu_context::WgpuContext,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        // テクスチャバインドグループレイアウトを作成
        let texture_bind_group_layout =
            crate::graphics::texture::Texture::create_bind_group_layout(&wgpu_context.device);

        // レンダリングパイプラインを作成
        let render_pipeline = wgpu_context.create_basic_pipeline(
            UNIT_SHADER,
            &[Vertex::desc(), UnitInstance::desc()],
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
                    label: Some("Unit Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // インデックスバッファを作成（2つの三角形で四角形を形成）
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Unit Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // インスタンスバッファを作成（初期容量）
        let max_instances = 1000; // 十分な数のユニットをサポート
        let instance_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Unit Instance Buffer"),
            size: (std::mem::size_of::<UnitInstance>() * max_instances) as wgpu::BufferAddress,
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
            max_instances,
        })
    }

    /// ユニットからインスタンスデータを更新
    fn update_instances(
        &mut self,
        units: &HashMap<u32, Unit>,
        options: &MapViewOptions,
        selected_unit_id: Option<u32>,
    ) {
        self.instances.clear();

        // タイルサイズを計算
        let tile_size = options.tile_size as f32 * options.zoom;

        // スクロール位置をタイル単位に変換
        let scroll_tile_x = if tile_size > 0.0 {
            options.scroll_x as f32 / tile_size
        } else {
            0.0
        };
        let scroll_tile_y = if tile_size > 0.0 {
            options.scroll_y as f32 / tile_size
        } else {
            0.0
        };

        // ビューポート内に表示されるタイルの範囲を計算
        let start_x = scroll_tile_x.max(0.0) as i32;
        let start_y = scroll_tile_y.max(0.0) as i32;
        let end_x = (scroll_tile_x + options.viewport_width as f32) as i32;
        let end_y = (scroll_tile_y + options.viewport_height as f32) as i32;

        // 表示範囲内のユニットをインスタンスとして追加
        for unit in units.values() {
            let x = unit.position.x;
            let y = unit.position.y;

            // ビューポート外のユニットはスキップ
            if x < start_x || x >= end_x || y < start_y || y >= end_y {
                continue;
            }

            // ユニットタイプに基づいてテクスチャ座標を設定
            let (tex_coords_min, tex_coords_max) = match unit.unit_type {
                UnitType::Infantry => ([0.0, 0.0], [0.2, 0.2]),
                UnitType::Cavalry => ([0.2, 0.0], [0.4, 0.2]),
                UnitType::Ranged => ([0.4, 0.0], [0.6, 0.2]),
                UnitType::Siege => ([0.6, 0.0], [0.8, 0.2]),
                UnitType::Support => ([0.8, 0.0], [1.0, 0.2]),
            };

            // 勢力IDとユニットタイプに基づいて色を設定
            let base_color = match unit.faction_id {
                1 => [0.0, 0.0, 1.0, 1.0], // 青（プレイヤー）
                2 => [0.0, 1.0, 0.0, 1.0], // 緑（同盟）
                3 => [1.0, 0.0, 0.0, 1.0], // 赤（敵対）
                _ => [0.7, 0.7, 0.7, 1.0], // グレー（中立）
            };

            // ユニットタイプに応じて色を調整（明るさを変える）
            let color = match unit.unit_type {
                UnitType::Infantry => [
                    base_color[0] * 1.0,
                    base_color[1] * 1.0,
                    base_color[2] * 1.0,
                    1.0,
                ], // 通常（歩兵）
                UnitType::Cavalry => [
                    base_color[0] * 1.2,
                    base_color[1] * 1.2,
                    base_color[2] * 1.2,
                    1.0,
                ], // 明るめ（騎兵）
                UnitType::Ranged => [
                    base_color[0] * 0.8,
                    base_color[1] * 0.8,
                    base_color[2] * 0.8,
                    1.0,
                ], // 暗め（弓兵）
                UnitType::Siege => [
                    base_color[0] * 0.6,
                    base_color[1] * 0.6,
                    base_color[2] * 0.6,
                    1.0,
                ], // さらに暗め（攻城兵器）
                UnitType::Support => [
                    base_color[0] * 1.4,
                    base_color[1] * 1.4,
                    base_color[2] * 1.4,
                    1.0,
                ], // さらに明るめ（支援ユニット）
            };

            // ユニットの位置を計算（タイルの中央に配置）
            let position = Vec3::new(x as f32, y as f32, 0.1); // Z値を少し上げてタイルの上に表示

            // モデル行列を計算
            let model_matrix = Mat4::from_scale_rotation_translation(
                Vec3::new(tile_size * 0.8, tile_size * 0.8, 1.0), // タイルより少し小さく
                glam::Quat::IDENTITY,
                position,
            );

            // 選択状態を設定
            let selected = if let Some(id) = selected_unit_id {
                if id == unit.id {
                    1
                } else {
                    0
                }
            } else {
                0
            };

            // インスタンスを追加
            self.instances.push(UnitInstance {
                model_matrix: model_matrix.to_cols_array_2d(),
                tex_coords_min,
                tex_coords_max,
                color,
                selected,
                _padding: [0, 0, 0],
            });
        }
    }

    /// ユニットをレンダリング
    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        units: &HashMap<u32, Unit>,
        _uniform_bind_group: &'a wgpu::BindGroup,
        options: &MapViewOptions,
        queue: &wgpu::Queue,
    ) {
        // ユニットからインスタンスデータを更新
        // 注: 実際の実装では、選択されたユニットIDをMapGUIから取得する必要がある
        self.update_instances(units, options, None);

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
}
