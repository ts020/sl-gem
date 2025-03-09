//! タイルレンダラー
//!
//! マップタイルのレンダリングを担当します。

use anyhow::Result;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::graphics::{
    renderer::{TileInstance, Vertex},
    shaders::TILE_SHADER,
};
use crate::gui::map_gui::MapViewOptions;
use model::{CellType, Map, MapPosition};

/// タイルレンダラー
pub struct TileRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    indices_len: u32,
    instances: Vec<TileInstance>,
    max_instances: usize,
}

impl TileRenderer {
    /// 新しいタイルレンダラーを作成
    pub fn new(
        wgpu_context: &crate::graphics::wgpu_context::WgpuContext,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        // テクスチャバインドグループレイアウトを作成
        let texture_bind_group_layout =
            crate::graphics::texture::Texture::create_bind_group_layout(&wgpu_context.device);

        // レンダリングパイプラインを作成
        let render_pipeline = wgpu_context.create_basic_pipeline(
            TILE_SHADER,
            &[Vertex::desc(), TileInstance::desc()],
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
                    label: Some("Tile Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // インデックスバッファを作成（2つの三角形で四角形を形成）
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Tile Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // インスタンスバッファを作成（初期容量）
        let max_instances = 10000; // 十分な数のタイルをサポート
        let instance_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tile Instance Buffer"),
            size: (std::mem::size_of::<TileInstance>() * max_instances) as wgpu::BufferAddress,
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

    /// マップからインスタンスデータを更新
    fn update_instances(&mut self, map: &Map, options: &MapViewOptions) {
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
        let end_x = (scroll_tile_x + options.viewport_width as f32).min(map.width as f32) as i32;
        let end_y = (scroll_tile_y + options.viewport_height as f32).min(map.height as f32) as i32;

        // 表示範囲内のタイルをインスタンスとして追加
        for y in start_y..end_y {
            for x in start_x..end_x {
                let pos = MapPosition::new(x, y);

                // セルタイプに基づいてテクスチャ座標を設定
                let (tex_coords_min, tex_coords_max) = match map.get_cell(&pos) {
                    Some(cell) => {
                        // 実際のテクスチャアトラスからUV座標を取得
                        match cell.cell_type {
                            CellType::Plain => ([0.0, 0.0], [0.125, 0.125]),
                            CellType::Forest => ([0.125, 0.0], [0.25, 0.125]),
                            CellType::Mountain => ([0.25, 0.0], [0.375, 0.125]),
                            CellType::Water => ([0.375, 0.0], [0.5, 0.125]),
                            CellType::Road => ([0.5, 0.0], [0.625, 0.125]),
                            CellType::City => ([0.625, 0.0], [0.75, 0.125]),
                            CellType::Base => ([0.75, 0.0], [0.875, 0.125]),
                        }
                    }
                    None => ([0.0, 0.0], [0.125, 0.125]), // デフォルトは平地
                };

                // タイルの位置を計算
                let position = Vec3::new(x as f32, y as f32, 0.0);

                // モデル行列を計算
                let model_matrix = Mat4::from_scale_rotation_translation(
                    Vec3::new(tile_size, tile_size, 1.0),
                    glam::Quat::IDENTITY,
                    position,
                );

                // セルタイプに応じて色を設定（シンプルに位置ベースのカラーリング）
                let color = match map.get_cell(&pos) {
                    Some(cell) => {
                        // デバッグ用：タイルの座標値から色を生成してチェッカーボードパターンを作る
                        // 注意: Rustの%演算子は符号付き整数に対して負の結果を返す可能性がある
                        // 例えば -1 % 2 は -1 になる
                        // 数学的なモジュロを得るにはrem_euclidを使用するべき
                        let raw_parity = (x + y) % 2;
                        let parity = if raw_parity < 0 {
                            (raw_parity + 2) % 2 // 負の場合は正の数に変換
                        } else {
                            raw_parity
                        };

                        // 詳細なデバッグ情報
                        println!(
                            "Position ({}, {}), raw_parity: {}, adjusted_parity: {}",
                            x, y, raw_parity, parity
                        );

                        // セルタイプと座標を出力
                        println!(
                            "セルタイプ: {:?}, 座標: ({}, {}), パリティ: {}",
                            cell.cell_type, x, y, parity
                        );

                        // セルタイプに基づいた色を設定
                        // 各セルタイプごとに異なる色を割り当て、区別しやすくする
                        let calculated_color = match cell.cell_type {
                            CellType::Plain => {
                                // 平地は赤/緑のチェッカーボードパターン
                                if parity == 0 {
                                    [1.0, 0.0, 0.0, 1.0] // 純赤色
                                } else {
                                    [0.0, 1.0, 0.0, 1.0] // 純緑色
                                }
                            }
                            CellType::Forest => [0.0, 0.6, 0.0, 1.0], // 深緑
                            CellType::Mountain => [0.5, 0.3, 0.0, 1.0], // 茶色
                            CellType::Water => [0.0, 0.0, 0.8, 1.0],  // 青色
                            CellType::Road => [0.7, 0.7, 0.0, 1.0],   // 黄色
                            CellType::City => [0.7, 0.7, 0.7, 1.0],   // 灰色
                            CellType::Base => [0.8, 0.0, 0.8, 1.0],   // 紫色
                        };

                        // 計算された色をデバッグ出力
                        println!(
                            "設定色: [{:.1}, {:.1}, {:.1}, {:.1}]",
                            calculated_color[0],
                            calculated_color[1],
                            calculated_color[2],
                            calculated_color[3]
                        );

                        calculated_color
                    }
                    None => {
                        println!("警告: 座標 ({}, {}) にセルが見つかりません", x, y);
                        [0.0, 0.0, 0.0, 1.0] // 黒色（デフォルト）
                    }
                };

                // インスタンスを追加
                self.instances.push(TileInstance {
                    model_matrix: model_matrix.to_cols_array_2d(),
                    tex_coords_min: tex_coords_min,
                    tex_coords_max: tex_coords_max,
                    color: color,
                });
            }
        }
    }

    /// タイルをレンダリング
    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        map: &Map,
        _uniform_bind_group: &'a wgpu::BindGroup,
        options: &MapViewOptions,
        queue: &wgpu::Queue,
    ) {
        // マップからインスタンスデータを更新
        self.update_instances(map, options);

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
