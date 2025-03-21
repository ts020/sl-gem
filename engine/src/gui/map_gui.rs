//! マップGUIコンポーネント
use crate::events::{EventBus, GameEvent};
use anyhow::Result;
use model::{Map, MapPosition, Unit};
use std::collections::HashMap;

/// マップGUIの表示オプション
#[derive(Debug, Clone)]
pub struct MapViewOptions {
    pub tile_size: u32,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub zoom: f32,
    pub show_grid: bool,
    pub viewport_width: u32,  // ビューポートの幅（タイル単位）
    pub viewport_height: u32, // ビューポートの高さ（タイル単位）
}

impl Default for MapViewOptions {
    fn default() -> Self {
        Self {
            tile_size: 32,
            scroll_x: 0,
            scroll_y: 0,
            zoom: 1.0,
            show_grid: true,
            viewport_width: 20,  // デフォルトのビューポート幅
            viewport_height: 15, // デフォルトのビューポート高さ
        }
    }
}

/// マップGUIコンポーネント
pub struct MapGUI {
    event_bus: EventBus,
    map: Option<Map>,
    units: HashMap<u32, Unit>,
    view_options: MapViewOptions,
    selected_position: Option<MapPosition>,
    selected_unit_id: Option<u32>,
    highlight_positions: Vec<MapPosition>,
}

impl MapGUI {
    /// 新しいMapGUIインスタンスを作成
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            event_bus,
            map: None,
            units: HashMap::new(),
            view_options: MapViewOptions::default(),
            selected_position: None,
            selected_unit_id: None,
            highlight_positions: Vec::new(),
        }
    }

    /// マップを設定
    pub fn set_map(&mut self, map: Map) {
        self.map = Some(map);
        self.publish_map_updated().ok();
    }

    /// マップを取得
    pub fn get_map(&self) -> Option<&Map> {
        self.map.as_ref()
    }

    /// ユニットを追加
    pub fn add_unit(&mut self, unit: Unit) {
        self.units.insert(unit.id, unit);
        self.publish_map_updated().ok();
    }

    /// ユニットを更新
    pub fn update_unit(&mut self, unit: Unit) -> bool {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.units.entry(unit.id) {
            e.insert(unit);
            self.publish_map_updated().ok();
            true
        } else {
            false
        }
    }

    /// ユニットを削除
    pub fn remove_unit(&mut self, unit_id: u32) -> bool {
        if self.units.remove(&unit_id).is_some() {
            if let Some(selected_id) = self.selected_unit_id {
                if selected_id == unit_id {
                    self.selected_unit_id = None;
                }
            }
            self.publish_map_updated().ok();
            true
        } else {
            false
        }
    }

    /// IDでユニットを取得
    pub fn get_unit(&self, unit_id: u32) -> Option<&Unit> {
        self.units.get(&unit_id)
    }

    /// 指定された位置にあるユニットを取得
    pub fn get_unit_at_position(&self, position: &MapPosition) -> Option<&Unit> {
        self.units
            .values()
            .find(|unit| unit.position.x == position.x && unit.position.y == position.y)
    }

    /// 表示オプションを設定
    pub fn set_view_options(&mut self, options: MapViewOptions) {
        self.view_options = options;
        self.publish_map_updated().ok();
    }

    /// 表示オプションを取得
    pub fn get_view_options(&self) -> &MapViewOptions {
        &self.view_options
    }

    /// マップをスクロール
    pub fn scroll(&mut self, dx: i32, dy: i32) {
        self.view_options.scroll_x += dx;
        self.view_options.scroll_y += dy;
        self.publish_map_updated().ok();
    }

    /// マップのズームを変更
    pub fn zoom(&mut self, factor: f32) {
        self.view_options.zoom *= factor;
        // ズーム値の制限
        self.view_options.zoom = self.view_options.zoom.clamp(0.25, 2.0);
        self.publish_map_updated().ok();
    }

    /// セルを選択
    pub fn select_position(&mut self, position: MapPosition) -> Result<()> {
        if let Some(map) = &self.map {
            if map.is_valid_position(&position) {
                self.selected_position = Some(position);
                // ユニット選択の確認
                let unit_at_position = self.get_unit_at_position(&position);
                if let Some(unit) = unit_at_position {
                    let unit_id = unit.id;
                    self.selected_unit_id = Some(unit_id);
                    self.publish_unit_selected(unit_id)?;
                } else {
                    self.selected_unit_id = None;
                }
                self.publish_position_selected(position)?;
                self.publish_map_updated()?;
                Ok(())
            } else {
                Err(anyhow::anyhow!("無効なマップ位置: {:?}", position))
            }
        } else {
            Err(anyhow::anyhow!("マップが設定されていません"))
        }
    }

    /// 選択位置を取得
    pub fn get_selected_position(&self) -> Option<MapPosition> {
        self.selected_position
    }

    /// 選択ユニットを取得
    pub fn get_selected_unit(&self) -> Option<&Unit> {
        self.selected_unit_id.and_then(|id| self.units.get(&id))
    }

    /// 選択解除
    pub fn clear_selection(&mut self) {
        self.selected_position = None;
        self.selected_unit_id = None;
        self.highlight_positions.clear();
        self.publish_map_updated().ok();
    }

    /// 特定の位置をハイライト表示
    pub fn highlight_positions(&mut self, positions: Vec<MapPosition>) {
        self.highlight_positions = positions;
        self.publish_map_updated().ok();
    }

    /// 現在ハイライト表示されている位置を取得
    pub fn get_highlight_positions(&self) -> &[MapPosition] {
        &self.highlight_positions
    }

    /// スクリーン座標からマップ座標への変換
    pub fn screen_to_map_position(&self, screen_x: i32, screen_y: i32) -> MapPosition {
        let tile_size = (self.view_options.tile_size as f32 * self.view_options.zoom) as i32;
        let map_x = (screen_x + self.view_options.scroll_x) / tile_size;
        let map_y = (screen_y + self.view_options.scroll_y) / tile_size;
        MapPosition { x: map_x, y: map_y }
    }

    /// マップ座標からスクリーン座標への変換
    pub fn map_to_screen_position(&self, map_x: i32, map_y: i32) -> (i32, i32) {
        let tile_size = (self.view_options.tile_size as f32 * self.view_options.zoom) as i32;
        let screen_x = map_x * tile_size - self.view_options.scroll_x;
        let screen_y = map_y * tile_size - self.view_options.scroll_y;
        (screen_x, screen_y)
    }

    /// マップ更新イベントを発行
    fn publish_map_updated(&self) -> Result<()> {
        self.event_bus.publish(
            "map_gui",
            GameEvent::Log {
                message: "マップ表示が更新されました".to_string(),
                level: crate::events::LogLevel::Info,
            },
        )
    }

    /// 位置選択イベントを発行
    fn publish_position_selected(&self, position: MapPosition) -> Result<()> {
        self.event_bus.publish(
            "map_gui",
            GameEvent::Log {
                message: format!("セル選択: ({}, {})", position.x, position.y),
                level: crate::events::LogLevel::Info,
            },
        )
    }

    /// ユニット選択イベントを発行
    fn publish_unit_selected(&self, unit_id: u32) -> Result<()> {
        self.event_bus.publish(
            "map_gui",
            GameEvent::Log {
                message: format!("ユニット選択: ID {}", unit_id),
                level: crate::events::LogLevel::Info,
            },
        )
    }

    /// マップGUIの描画（実際の描画はレンダリングシステムに任せる）
    pub fn render(&self) {
        // このメソッドは、将来的にはレンダリングシステムにマップGUIの状態を提供します
        // 現在は抽象的なインターフェースとしてのみ存在しています
    }

    /// ASCIIアートとしてマップを表示する
    pub fn render_ascii(&self) -> String {
        if let Some(map) = &self.map {
            let mut output = String::new();

            // スクロール位置をタイル単位に変換（小数点以下切り捨て）
            let scaled_tile_size =
                (self.view_options.tile_size as f32 * self.view_options.zoom) as i32;
            let scroll_tile_x = if scaled_tile_size > 0 {
                self.view_options.scroll_x / scaled_tile_size
            } else {
                0
            };
            let scroll_tile_y = if scaled_tile_size > 0 {
                self.view_options.scroll_y / scaled_tile_size
            } else {
                0
            };

            // ビューポート内に表示されるタイルの範囲を計算
            let start_x = scroll_tile_x.max(0);
            let start_y = scroll_tile_y.max(0);
            let end_x =
                (scroll_tile_x + self.view_options.viewport_width as i32).min(map.width as i32);
            let end_y =
                (scroll_tile_y + self.view_options.viewport_height as i32).min(map.height as i32);

            // スクロール情報を表示
            output.push_str(&format!(
                "スクロール位置: ({}, {}) タイル\n",
                start_x, start_y
            ));
            output.push_str(&format!(
                "表示範囲: {}×{} タイル\n",
                end_x - start_x,
                end_y - start_y
            ));

            // ヘッダー行（X座標）を追加
            output.push_str("   ");
            for x in start_x..end_x {
                output.push_str(&format!("{:2}", x % 10));
            }
            output.push('\n');

            // 境界線
            output.push_str("  +");
            for _ in start_x..end_x {
                output.push_str("--");
            }
            output.push_str("+\n");

            for y in start_y..end_y {
                // Y座標を追加
                output.push_str(&format!("{:2}|", y % 10));

                for x in start_x..end_x {
                    let pos = MapPosition::new(x, y);
                    let is_selected = self
                        .selected_position
                        .is_some_and(|selected| selected.x == x && selected.y == y);
                    let is_highlighted = self
                        .highlight_positions
                        .iter()
                        .any(|p| p.x == x && p.y == y);

                    // ユニットの確認
                    let unit_at_pos = self.get_unit_at_position(&pos);

                    // セルタイプに基づいて文字を選択
                    let mut symbol = match map.get_cell(&pos) {
                        Some(cell) => match cell.cell_type {
                            model::CellType::Plain => ".",
                            model::CellType::Forest => "T",
                            model::CellType::Mountain => "^",
                            model::CellType::Water => "~",
                            model::CellType::Road => "=",
                            model::CellType::City => "C",
                            model::CellType::Base => "B",
                        },
                        None => " ",
                    }
                    .to_string();

                    // ユニットがある場合はユニットの文字を優先
                    if let Some(unit) = unit_at_pos {
                        symbol = match unit.unit_type {
                            model::UnitType::Infantry => "I",
                            model::UnitType::Cavalry => "K",
                            model::UnitType::Ranged => "R",
                            model::UnitType::Siege => "S",
                            model::UnitType::Support => "U",
                        }
                        .to_string();

                        // ユニットの所有勢力によって色分けできないので、勢力IDを数字で表現（将来的にはANSIカラーコードなどで色付け可能）
                        if unit.faction_id > 0 {
                            symbol = format!("{}", unit.faction_id);
                        }
                    }

                    // 選択または強調表示の装飾
                    if is_selected {
                        symbol = format!("[{}]", symbol);
                    } else if is_highlighted {
                        symbol = format!("*{}*", symbol);
                    } else {
                        symbol = format!(" {} ", symbol);
                    }

                    output.push_str(&symbol);
                }

                output.push_str("|\n");
            }

            // 下部境界線
            output.push_str("  +");
            for _ in start_x..end_x {
                output.push_str("--");
            }
            output.push_str("+\n");

            output
        } else {
            "マップが設定されていません。".to_string()
        }
    }

    /// コンソールにASCIIアートとしてマップを表示する
    pub fn print_ascii_map(&self) {
        println!("{}", self.render_ascii());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{Cell, CellType, UnitType};

    fn create_test_map() -> Map {
        let mut map = Map::new(10, 10);

        // いくつかのセルを設定
        for x in 0..10 {
            for y in 0..10 {
                let cell_type = match (x + y) % 3 {
                    0 => CellType::Plain,
                    1 => CellType::Forest,
                    _ => CellType::Mountain,
                };
                map.set_cell(MapPosition::new(x, y), Cell::new(cell_type));
            }
        }

        map
    }

    fn create_test_unit(id: u32, x: i32, y: i32) -> Unit {
        Unit::new(
            id,
            format!("テストユニット{}", id),
            UnitType::Infantry,
            1, // faction_id
            MapPosition::new(x, y),
        )
    }

    #[test]
    fn test_map_gui_initialization() {
        let event_bus = EventBus::new();
        let map_gui = MapGUI::new(event_bus);

        assert!(map_gui.get_map().is_none());
        assert_eq!(map_gui.units.len(), 0);
        assert!(map_gui.selected_position.is_none());
        assert!(map_gui.selected_unit_id.is_none());
    }

    #[test]
    fn test_map_setting() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        let map = create_test_map();
        map_gui.set_map(map);

        assert!(map_gui.get_map().is_some());
        if let Some(map) = map_gui.get_map() {
            assert_eq!(map.width, 10);
            assert_eq!(map.height, 10);
        }
    }

    #[test]
    fn test_unit_management() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        // ユニット追加
        let unit1 = create_test_unit(1, 3, 4);
        let unit2 = create_test_unit(2, 5, 6);

        map_gui.add_unit(unit1);
        map_gui.add_unit(unit2);

        assert_eq!(map_gui.units.len(), 2);

        // ユニット取得
        let retrieved_unit = map_gui.get_unit(1);
        assert!(retrieved_unit.is_some());
        if let Some(unit) = retrieved_unit {
            assert_eq!(unit.id, 1);
            assert_eq!(unit.position.x, 3);
            assert_eq!(unit.position.y, 4);
        }

        // ユニット更新
        let mut updated_unit = create_test_unit(1, 7, 8);
        updated_unit.health = 80;
        assert!(map_gui.update_unit(updated_unit));

        let updated = map_gui.get_unit(1);
        assert!(updated.is_some());
        if let Some(unit) = updated {
            assert_eq!(unit.position.x, 7);
            assert_eq!(unit.position.y, 8);
            assert_eq!(unit.health, 80);
        }

        // ユニット削除
        assert!(map_gui.remove_unit(1));
        assert_eq!(map_gui.units.len(), 1);
        assert!(map_gui.get_unit(1).is_none());

        // 存在しないユニットの削除は失敗する
        assert!(!map_gui.remove_unit(999));
    }

    #[test]
    fn test_position_selection() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        let map = create_test_map();
        map_gui.set_map(map);

        // 有効な位置の選択
        let pos = MapPosition::new(5, 5);
        assert!(map_gui.select_position(pos).is_ok());
        assert_eq!(map_gui.get_selected_position(), Some(pos));

        // 無効な位置の選択
        let invalid_pos = MapPosition::new(20, 20);
        assert!(map_gui.select_position(invalid_pos).is_err());

        // 選択解除
        map_gui.clear_selection();
        assert!(map_gui.get_selected_position().is_none());
    }

    #[test]
    fn test_unit_selection() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        let map = create_test_map();
        map_gui.set_map(map);

        let unit = create_test_unit(1, 3, 4);
        map_gui.add_unit(unit);

        // ユニットがいる位置を選択
        assert!(map_gui.select_position(MapPosition::new(3, 4)).is_ok());
        assert_eq!(map_gui.selected_unit_id, Some(1));

        let selected_unit = map_gui.get_selected_unit();
        assert!(selected_unit.is_some());
        if let Some(unit) = selected_unit {
            assert_eq!(unit.id, 1);
        }

        // ユニットがいない位置を選択
        assert!(map_gui.select_position(MapPosition::new(5, 5)).is_ok());
        assert_eq!(map_gui.selected_unit_id, None);
        assert!(map_gui.get_selected_unit().is_none());
    }

    #[test]
    fn test_coordinate_conversion() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        // デフォルトビュー設定でのテスト
        let map_pos = MapPosition::new(3, 4);
        let (screen_x, screen_y) = map_gui.map_to_screen_position(map_pos.x, map_pos.y);
        let converted_pos = map_gui.screen_to_map_position(screen_x, screen_y);

        assert_eq!(converted_pos.x, map_pos.x);
        assert_eq!(converted_pos.y, map_pos.y);

        // スクロール後のテスト
        map_gui.scroll(100, 50);
        let (scrolled_x, scrolled_y) = map_gui.map_to_screen_position(map_pos.x, map_pos.y);
        assert_ne!((scrolled_x, scrolled_y), (screen_x, screen_y));

        // ズーム後のテスト
        map_gui.view_options.scroll_x = 0;
        map_gui.view_options.scroll_y = 0;
        map_gui.zoom(2.0);
        let (zoomed_x, zoomed_y) = map_gui.map_to_screen_position(map_pos.x, map_pos.y);
        assert_ne!((zoomed_x, zoomed_y), (screen_x, screen_y));
    }

    #[test]
    fn test_highlight_positions() {
        let event_bus = EventBus::new();
        let mut map_gui = MapGUI::new(event_bus);

        let positions = vec![
            MapPosition::new(1, 1),
            MapPosition::new(2, 2),
            MapPosition::new(3, 3),
        ];

        map_gui.highlight_positions(positions.clone());

        let highlights = map_gui.get_highlight_positions();
        assert_eq!(highlights.len(), 3);

        for (i, pos) in positions.iter().enumerate() {
            assert_eq!(highlights[i].x, pos.x);
            assert_eq!(highlights[i].y, pos.y);
        }

        // 選択解除でハイライトもクリアされる
        map_gui.clear_selection();
        assert!(map_gui.get_highlight_positions().is_empty());
    }
}
