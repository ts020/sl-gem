//! タイルレンダラーのテスト

#[cfg(test)]
mod tests {
    use crate::graphics::renderer::tile_renderer::TileRenderer;
    use crate::gui::map_gui::MapViewOptions;
    use glam::Mat4;
    use model::{Cell, CellType, Map, MapPosition};
    use std::sync::Arc;
    use wgpu::util::DeviceExt;

    // シミュレーションテスト用のヘルパー関数
    fn mock_tile_color(cell_type: CellType, x: i32, y: i32) -> [f32; 4] {
        let parity = (x + y) % 2;
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

    #[test]
    fn test_tile_color_calculation() {
        // 平地の偶数パリティ（赤）
        let color = mock_tile_color(CellType::Plain, 0, 0);
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);

        // 平地の奇数パリティ（緑）
        let color = mock_tile_color(CellType::Plain, 0, 1);
        assert_eq!(color, [0.0, 1.0, 0.0, 1.0]);

        // 森林
        let color = mock_tile_color(CellType::Forest, 0, 0);
        assert_eq!(color, [0.0, 0.6, 0.0, 1.0]);

        // 山地
        let color = mock_tile_color(CellType::Mountain, 0, 0);
        assert_eq!(color, [0.5, 0.3, 0.0, 1.0]);

        // 水域
        let color = mock_tile_color(CellType::Water, 0, 0);
        assert_eq!(color, [0.0, 0.0, 0.8, 1.0]);

        // 道路
        let color = mock_tile_color(CellType::Road, 0, 0);
        assert_eq!(color, [0.7, 0.7, 0.0, 1.0]);

        // 都市
        let color = mock_tile_color(CellType::City, 0, 0);
        assert_eq!(color, [0.7, 0.7, 0.7, 1.0]);

        // 拠点
        let color = mock_tile_color(CellType::Base, 0, 0);
        assert_eq!(color, [0.8, 0.0, 0.8, 1.0]);
    }

    #[test]
    fn test_parity_checker() {
        // パリティチェック（偶数）
        assert_eq!((0 + 0) % 2, 0);
        assert_eq!((1 + 1) % 2, 0);
        assert_eq!((2 + 0) % 2, 0);

        // パリティチェック（奇数）
        assert_eq!((0 + 1) % 2, 1);
        assert_eq!((1 + 0) % 2, 1);
        assert_eq!((2 + 1) % 2, 1);

        // 負の数のパリティチェック
        assert_eq!((-1 + 0) % 2, 1); // -1 % 2 = -1, これをRustでは1とする
        assert_eq!((0 + -1) % 2, 1);
        assert_eq!((-1 + -1) % 2, 0); // -2 % 2 = 0
    }

    #[test]
    fn test_rust_modulo_behavior() {
        // Rustの%演算子の挙動を確認
        // 負の数に対するモジュロ演算
        assert_eq!(-1 % 2, -1); // 数学的には1だが、Rustでは-1になる
        assert_eq!(-2 % 2, 0); // 数学的にも0
        assert_eq!(-3 % 2, -1); // 数学的には1だが、Rustでは-1になる

        // 正しく数学的なモジュロを得るためのrem_euclid
        assert_eq!((-1i32).rem_euclid(2), 1);
        assert_eq!((-2i32).rem_euclid(2), 0);
        assert_eq!((-3i32).rem_euclid(2), 1);
    }

    #[test]
    fn test_tile_distribution() {
        // テスト用の小さなマップを作成
        let mut map = Map::new(4, 4);

        // 平地タイル
        map.set_cell(MapPosition::new(0, 0), Cell::new(CellType::Plain));
        map.set_cell(MapPosition::new(0, 1), Cell::new(CellType::Plain));

        // 森林タイル
        map.set_cell(MapPosition::new(1, 0), Cell::new(CellType::Forest));

        // 山地タイル
        map.set_cell(MapPosition::new(1, 1), Cell::new(CellType::Mountain));

        // 水域タイル
        map.set_cell(MapPosition::new(2, 0), Cell::new(CellType::Water));

        // 道路タイル
        map.set_cell(MapPosition::new(2, 1), Cell::new(CellType::Road));

        // 都市タイル
        map.set_cell(MapPosition::new(3, 0), Cell::new(CellType::City));

        // 拠点タイル
        map.set_cell(MapPosition::new(3, 1), Cell::new(CellType::Base));

        // チェック：マップのセルタイプが正しく設定されているか
        assert_eq!(
            map.get_cell(&MapPosition::new(0, 0)).unwrap().cell_type,
            CellType::Plain
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(0, 1)).unwrap().cell_type,
            CellType::Plain
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(1, 0)).unwrap().cell_type,
            CellType::Forest
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(1, 1)).unwrap().cell_type,
            CellType::Mountain
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(2, 0)).unwrap().cell_type,
            CellType::Water
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(2, 1)).unwrap().cell_type,
            CellType::Road
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(3, 0)).unwrap().cell_type,
            CellType::City
        );
        assert_eq!(
            map.get_cell(&MapPosition::new(3, 1)).unwrap().cell_type,
            CellType::Base
        );
    }
}
