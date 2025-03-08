use std::collections::HashMap;

/// 2D座標を表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapPosition {
    pub x: i32,
    pub y: i32,
}

impl MapPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// 指定された方向に移動した新しい位置を返す
    pub fn moved(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// 2点間のマンハッタン距離を計算
    pub fn manhattan_distance(&self, other: &MapPosition) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }
}

/// マップのセルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Plain,    // 平地
    Forest,   // 森
    Mountain, // 山
    Water,    // 水域
    Road,     // 道路
    City,     // 都市
    Base,     // 拠点
}

impl CellType {
    /// セルタイプの移動コストを返す
    pub fn movement_cost(&self) -> u32 {
        match self {
            CellType::Plain => 1,
            CellType::Forest => 2,
            CellType::Mountain => 3,
            CellType::Water => u32::MAX, // 通過不可
            CellType::Road => 1,
            CellType::City => 1,
            CellType::Base => 1,
        }
    }

    /// 防御修正値を返す (%)
    pub fn defense_modifier(&self) -> i32 {
        match self {
            CellType::Plain => 0,
            CellType::Forest => 20,
            CellType::Mountain => 40,
            CellType::Water => 0,
            CellType::Road => -10,
            CellType::City => 30,
            CellType::Base => 50,
        }
    }
}

/// マップのセル
#[derive(Debug, Clone)]
pub struct Cell {
    pub cell_type: CellType,
    pub faction_id: Option<u32>, // 所有勢力ID（ある場合）
}

impl Cell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            faction_id: None,
        }
    }

    pub fn with_faction(cell_type: CellType, faction_id: u32) -> Self {
        Self {
            cell_type,
            faction_id: Some(faction_id),
        }
    }
}

/// ゲームマップ
#[derive(Debug, Clone)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    cells: HashMap<MapPosition, Cell>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cells: HashMap::new(),
        }
    }

    /// 指定された位置にセルを設定
    pub fn set_cell(&mut self, pos: MapPosition, cell: Cell) {
        if self.is_valid_position(&pos) {
            self.cells.insert(pos, cell);
        }
    }

    /// 指定された位置のセルを取得
    pub fn get_cell(&self, pos: &MapPosition) -> Option<&Cell> {
        if self.is_valid_position(pos) {
            self.cells.get(pos)
        } else {
            None
        }
    }

    /// 指定された位置が有効かどうかを検証
    pub fn is_valid_position(&self, pos: &MapPosition) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as i32 && pos.y < self.height as i32
    }

    /// 指定された位置の隣接セルの位置を取得
    pub fn get_adjacent_positions(&self, pos: &MapPosition) -> Vec<MapPosition> {
        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)]; // 上、右、下、左

        directions
            .iter()
            .map(|(dx, dy)| pos.moved(*dx, *dy))
            .filter(|new_pos| self.is_valid_position(new_pos))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = MapPosition::new(5, 10);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);

        let moved = pos.moved(2, -3);
        assert_eq!(moved.x, 7);
        assert_eq!(moved.y, 7);

        let distance = pos.manhattan_distance(&moved);
        assert_eq!(distance, 5);
    }

    #[test]
    fn test_cell_type() {
        assert_eq!(CellType::Plain.movement_cost(), 1);
        assert_eq!(CellType::Forest.movement_cost(), 2);
        assert_eq!(CellType::Mountain.movement_cost(), 3);

        assert_eq!(CellType::Forest.defense_modifier(), 20);
        assert_eq!(CellType::Mountain.defense_modifier(), 40);
    }

    #[test]
    fn test_map_basic() {
        let mut map = Map::new(10, 10);

        let pos = MapPosition::new(5, 5);
        map.set_cell(pos, Cell::new(CellType::Plain));

        let cell = map.get_cell(&pos).unwrap();
        assert_eq!(cell.cell_type, CellType::Plain);
        assert_eq!(cell.faction_id, None);

        let invalid_pos = MapPosition::new(20, 20);
        assert!(map.get_cell(&invalid_pos).is_none());
    }

    #[test]
    fn test_map_adjacency() {
        let mut map = Map::new(5, 5);

        // マップ中央のセル
        let center = MapPosition::new(2, 2);
        map.set_cell(center, Cell::new(CellType::Plain));

        // マップの端のセル
        let edge = MapPosition::new(0, 0);
        map.set_cell(edge, Cell::new(CellType::Forest));

        let center_adjacent = map.get_adjacent_positions(&center);
        assert_eq!(center_adjacent.len(), 4); // 四方向すべて有効

        let edge_adjacent = map.get_adjacent_positions(&edge);
        assert_eq!(edge_adjacent.len(), 2); // 右と下のみ有効
    }
}
