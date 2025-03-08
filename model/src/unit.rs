use crate::map::Position;

/// ユニットの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Infantry, // 歩兵
    Cavalry,  // 騎兵
    Ranged,   // 遠距離
    Siege,    // 攻城
    Support,  // 支援
}

impl UnitType {
    /// ユニットの基本移動力を返す
    pub fn base_movement(&self) -> u32 {
        match self {
            UnitType::Infantry => 3,
            UnitType::Cavalry => 5,
            UnitType::Ranged => 2,
            UnitType::Siege => 1,
            UnitType::Support => 2,
        }
    }

    /// ユニットの基本攻撃力を返す
    pub fn base_attack(&self) -> u32 {
        match self {
            UnitType::Infantry => 10,
            UnitType::Cavalry => 12,
            UnitType::Ranged => 8,
            UnitType::Siege => 15,
            UnitType::Support => 4,
        }
    }

    /// ユニットの基本防御力を返す
    pub fn base_defense(&self) -> u32 {
        match self {
            UnitType::Infantry => 10,
            UnitType::Cavalry => 8,
            UnitType::Ranged => 6,
            UnitType::Siege => 5,
            UnitType::Support => 7,
        }
    }
}

/// ユニットの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitStatus {
    Idle,      // 待機
    Moving,    // 移動中
    Attacking, // 攻撃中
    Defending, // 防御中
    Exhausted, // 行動済み
    Wounded,   // 負傷
}

/// ゲーム内のユニット
#[derive(Debug, Clone)]
pub struct Unit {
    pub id: u32,
    pub name: String,
    pub unit_type: UnitType,
    pub faction_id: u32,
    pub position: Position,
    pub health: u32,
    pub experience: u32,
    pub status: UnitStatus,
    // 追加の属性
    pub movement_points: u32,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
}

impl Unit {
    pub fn new(
        id: u32,
        name: String,
        unit_type: UnitType,
        faction_id: u32,
        position: Position,
    ) -> Self {
        let movement_points = unit_type.base_movement();

        Self {
            id,
            name,
            unit_type,
            faction_id,
            position,
            health: 100,
            experience: 0,
            status: UnitStatus::Idle,
            movement_points,
            attack_bonus: 0,
            defense_bonus: 0,
        }
    }

    /// ユニットの現在の攻撃力を計算
    pub fn attack_power(&self) -> u32 {
        let base = self.unit_type.base_attack();
        let exp_bonus = (self.experience / 100) as i32; // 経験値ごとに攻撃力ボーナス
        let health_factor = self.health as f32 / 100.0; // 体力による減衰

        let total = (base as i32 + self.attack_bonus + exp_bonus) as f32 * health_factor;
        total.max(1.0) as u32 // 最低でも1の攻撃力を確保
    }

    /// ユニットの現在の防御力を計算
    pub fn defense_power(&self) -> u32 {
        let base = self.unit_type.base_defense();
        let exp_bonus = (self.experience / 150) as i32; // 経験値ごとに防御力ボーナス
        let health_factor = self.health as f32 / 100.0; // 体力による減衰

        let total = (base as i32 + self.defense_bonus + exp_bonus) as f32 * health_factor;
        total.max(1.0) as u32 // 最低でも1の防御力を確保
    }

    /// ユニットの移動
    pub fn move_to(&mut self, new_position: Position, cost: u32) -> bool {
        if self.movement_points >= cost {
            self.position = new_position;
            self.movement_points -= cost;
            self.status = if self.movement_points == 0 {
                UnitStatus::Exhausted
            } else {
                UnitStatus::Moving
            };
            true
        } else {
            false
        }
    }

    /// ターン開始時のリセット
    pub fn reset_for_new_turn(&mut self) {
        self.movement_points = self.unit_type.base_movement();
        if self.status == UnitStatus::Exhausted {
            self.status = UnitStatus::Idle;
        }
    }

    /// ダメージを受ける
    pub fn take_damage(&mut self, amount: u32) -> bool {
        let actual_damage = amount.min(self.health);
        self.health -= actual_damage;

        if self.health == 0 {
            // ユニットが倒された
            false
        } else if self.health < 30 {
            // 重傷
            self.status = UnitStatus::Wounded;
            true
        } else {
            true
        }
    }

    /// 経験値を獲得
    pub fn gain_experience(&mut self, amount: u32) {
        self.experience += amount;
        // レベルアップのロジックは別途実装
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_type_stats() {
        assert_eq!(UnitType::Infantry.base_movement(), 3);
        assert_eq!(UnitType::Cavalry.base_movement(), 5);

        assert_eq!(UnitType::Infantry.base_attack(), 10);
        assert_eq!(UnitType::Siege.base_attack(), 15);

        assert_eq!(UnitType::Infantry.base_defense(), 10);
        assert_eq!(UnitType::Ranged.base_defense(), 6);
    }

    #[test]
    fn test_unit_creation() {
        let position = Position::new(5, 5);
        let unit = Unit::new(1, "テスト歩兵".to_string(), UnitType::Infantry, 1, position);

        assert_eq!(unit.id, 1);
        assert_eq!(unit.name, "テスト歩兵");
        assert_eq!(unit.unit_type, UnitType::Infantry);
        assert_eq!(unit.faction_id, 1);
        assert_eq!(unit.position.x, 5);
        assert_eq!(unit.position.y, 5);
        assert_eq!(unit.health, 100);
        assert_eq!(unit.experience, 0);
        assert_eq!(unit.status, UnitStatus::Idle);
        assert_eq!(unit.movement_points, 3); // 歩兵の基本移動力
    }

    #[test]
    fn test_unit_movement() {
        let start_pos = Position::new(1, 1);
        let mut unit = Unit::new(1, "テスト騎兵".to_string(), UnitType::Cavalry, 1, start_pos);

        assert_eq!(unit.movement_points, 5); // 騎兵の基本移動力

        // 移動の成功
        let new_pos = Position::new(3, 3);
        assert!(unit.move_to(new_pos, 3));
        assert_eq!(unit.position.x, 3);
        assert_eq!(unit.position.y, 3);
        assert_eq!(unit.movement_points, 2);
        assert_eq!(unit.status, UnitStatus::Moving);

        // 移動ポイントをすべて使い切る移動
        let final_pos = Position::new(4, 4);
        assert!(unit.move_to(final_pos, 2));
        assert_eq!(unit.position.x, 4);
        assert_eq!(unit.position.y, 4);
        assert_eq!(unit.movement_points, 0);
        assert_eq!(unit.status, UnitStatus::Exhausted);

        // 移動ポイントが足りない場合
        let impossible_pos = Position::new(5, 5);
        assert!(!unit.move_to(impossible_pos, 1));
        assert_eq!(unit.position.x, 4); // 位置は変わらない
        assert_eq!(unit.position.y, 4);
    }

    #[test]
    fn test_unit_damage() {
        let position = Position::new(0, 0);
        let mut unit = Unit::new(1, "テスト歩兵".to_string(), UnitType::Infantry, 1, position);

        // 軽いダメージ
        assert!(unit.take_damage(20));
        assert_eq!(unit.health, 80);
        assert_eq!(unit.status, UnitStatus::Idle); // ステータスは変わらない

        // 重いダメージ（負傷状態になる）
        assert!(unit.take_damage(60));
        assert_eq!(unit.health, 20);
        assert_eq!(unit.status, UnitStatus::Wounded);

        // 致命的なダメージ
        assert!(!unit.take_damage(30)); // falseを返す（ユニットが倒された）
        assert_eq!(unit.health, 0);
    }

    #[test]
    fn test_unit_reset() {
        let position = Position::new(0, 0);
        let mut unit = Unit::new(1, "テスト歩兵".to_string(), UnitType::Infantry, 1, position);

        // 移動ポイントを消費して疲労状態に
        assert!(unit.move_to(Position::new(1, 1), 3));
        assert_eq!(unit.movement_points, 0);
        assert_eq!(unit.status, UnitStatus::Exhausted);

        // ターンリセット
        unit.reset_for_new_turn();
        assert_eq!(unit.movement_points, 3); // 歩兵の基本移動力に戻る
        assert_eq!(unit.status, UnitStatus::Idle); // 待機状態に戻る
    }

    #[test]
    fn test_unit_power_calculation() {
        let position = Position::new(0, 0);
        let mut unit = Unit::new(1, "テスト歩兵".to_string(), UnitType::Infantry, 1, position);

        // 初期状態
        assert_eq!(unit.attack_power(), 10); // 基本攻撃力
        assert_eq!(unit.defense_power(), 10); // 基本防御力

        // ボーナス追加
        unit.attack_bonus = 5;
        unit.defense_bonus = 3;
        assert_eq!(unit.attack_power(), 15);
        assert_eq!(unit.defense_power(), 13);

        // 体力減少の影響
        unit.health = 50;
        assert_eq!(unit.attack_power(), 7); // (10 + 5) * 0.5 = 7.5 → 7
        assert_eq!(unit.defense_power(), 6); // (10 + 3) * 0.5 = 6.5 → 6

        // 経験値の影響
        unit.health = 100; // 体力を戻す
        unit.experience = 300;
        assert_eq!(unit.attack_power(), 18); // 10 + 5 + 3 = 18
        assert_eq!(unit.defense_power(), 15); // 10 + 3 + 2 = 15
    }
}
