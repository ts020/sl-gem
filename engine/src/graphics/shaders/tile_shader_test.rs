//! タイルシェーダーのテスト

#[cfg(test)]
mod tests {
    use crate::graphics::renderer::TileInstance;
    use glam::{Mat4, Vec2, Vec3};
    use model::{CellType, MapPosition};

    // パリティテスト関数
    fn calculate_parity(x: i32, y: i32) -> u32 {
        ((x + y) % 2) as u32
    }

    // 座標からインスタンスの色を計算する関数
    fn calculate_color_from_position(cell_type: CellType, x: i32, y: i32) -> [f32; 4] {
        let parity = calculate_parity(x, y);

        match cell_type {
            CellType::Plain => {
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
        }
    }

    // タイルインスタンスのセットアップ関数
    fn setup_tile_instance(position: MapPosition, cell_type: CellType) -> TileInstance {
        let x = position.x;
        let y = position.y;
        let tile_size = 32.0;

        // ワールド座標に変換
        let world_x = x as f32 * tile_size;
        let world_y = y as f32 * tile_size;

        // モデル行列を作成
        let model_matrix = Mat4::from_translation(Vec3::new(world_x, world_y, 0.0));

        // テクスチャ座標範囲（実際のアトラスに合わせて調整が必要）
        let tex_coords_min = Vec2::new(0.0, 0.0);
        let tex_coords_max = Vec2::new(1.0, 1.0);

        // 色を計算
        let color = calculate_color_from_position(cell_type, x, y);

        TileInstance {
            model_matrix: model_matrix.to_cols_array_2d(),
            tex_coords_min: tex_coords_min.into(),
            tex_coords_max: tex_coords_max.into(),
            color,
        }
    }

    // カラー計算のテスト
    #[test]
    fn test_color_calculation() {
        // パリティ0（偶数）の赤色平地
        let pos = MapPosition::new(0, 0);
        let instance = setup_tile_instance(pos, CellType::Plain);
        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);

        // パリティ1（奇数）の緑色平地
        let pos = MapPosition::new(0, 1);
        let instance = setup_tile_instance(pos, CellType::Plain);
        assert_eq!(instance.color, [0.0, 1.0, 0.0, 1.0]);
    }

    // シェーダー入力値の検証
    #[test]
    fn test_shader_input_values() {
        // いくつかの異なるタイル種類に対してインスタンスを作成
        let positions = [
            (MapPosition::new(0, 0), CellType::Plain),
            (MapPosition::new(1, 0), CellType::Forest),
            (MapPosition::new(0, 1), CellType::Water),
            (MapPosition::new(1, 1), CellType::Mountain),
        ];

        for (pos, cell_type) in positions.iter() {
            let instance = setup_tile_instance(*pos, *cell_type);

            // モデル行列が正しく設定されているか
            let expected_x = pos.x as f32 * 32.0;
            let expected_y = pos.y as f32 * 32.0;
            assert_eq!(instance.model_matrix[3][0], expected_x);
            assert_eq!(instance.model_matrix[3][1], expected_y);

            // 色が正しく設定されているか
            let expected_color = calculate_color_from_position(*cell_type, pos.x, pos.y);
            assert_eq!(instance.color, expected_color);
        }
    }

    // フラグメントシェーダーの挙動シミュレーション
    #[test]
    fn test_fragment_shader_simulation() {
        // ダミーテクスチャカラー（白色）
        let tex_color = [1.0, 1.0, 1.0, 1.0];

        // 平地タイル（パリティ0）
        let pos = MapPosition::new(0, 0);
        let instance = setup_tile_instance(pos, CellType::Plain);

        // フラグメントシェーダーのロジックをシミュレート
        let final_color = [
            tex_color[0] * instance.color[0],
            tex_color[1] * instance.color[1],
            tex_color[2] * instance.color[2],
            tex_color[3] * instance.color[3],
        ];

        // 期待される結果: 白(1,1,1,1) * 赤(1,0,0,1) = 赤(1,0,0,1)
        assert_eq!(final_color, [1.0, 0.0, 0.0, 1.0]);

        // 平地タイル（パリティ1）
        let pos = MapPosition::new(0, 1);
        let instance = setup_tile_instance(pos, CellType::Plain);

        // フラグメントシェーダーのロジックをシミュレート
        let final_color = [
            tex_color[0] * instance.color[0],
            tex_color[1] * instance.color[1],
            tex_color[2] * instance.color[2],
            tex_color[3] * instance.color[3],
        ];

        // 期待される結果: 白(1,1,1,1) * 緑(0,1,0,1) = 緑(0,1,0,1)
        assert_eq!(final_color, [0.0, 1.0, 0.0, 1.0]);
    }
}
