use std::collections::HashMap;

/// 勢力の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionType {
    Player,      // プレイヤー
    Ally,        // 同盟
    Neutral,     // 中立
    Rival,       // 敵対
    Independent, // 独立
}

/// 勢力間の関係性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    Friendly, // 友好
    Neutral,  // 中立
    Hostile,  // 敵対
    Allied,   // 同盟
    AtWar,    // 交戦
}

impl Relationship {
    /// 関係性に基づくコスト修正を返す (交渉/通行など)
    pub fn cost_modifier(&self) -> f32 {
        match self {
            Relationship::Friendly => 0.8,
            Relationship::Neutral => 1.0,
            Relationship::Hostile => 1.5,
            Relationship::Allied => 0.5,
            Relationship::AtWar => 2.0,
        }
    }

    /// 関係性が通行可能かどうか
    pub fn allows_passage(&self) -> bool {
        !matches!(self, Relationship::Hostile | Relationship::AtWar)
    }

    /// 関係性が攻撃可能かどうか
    pub fn allows_attack(&self) -> bool {
        matches!(self, Relationship::Hostile | Relationship::AtWar)
    }
}

/// ゲーム内の勢力
#[derive(Debug, Clone)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub faction_type: FactionType,
    pub color: (u8, u8, u8), // RGB
    pub gold: u32,
    pub diplomatic_points: u32,
    pub relationships: HashMap<u32, Relationship>, // 他の勢力IDとの関係
}

impl Faction {
    pub fn new(id: u32, name: String, faction_type: FactionType, color: (u8, u8, u8)) -> Self {
        Self {
            id,
            name,
            faction_type,
            color,
            gold: 100,
            diplomatic_points: 0,
            relationships: HashMap::new(),
        }
    }

    /// 別の勢力との関係を設定
    pub fn set_relationship(&mut self, other_id: u32, relationship: Relationship) {
        self.relationships.insert(other_id, relationship);
    }

    /// 別の勢力との関係を取得（デフォルトは中立）
    pub fn get_relationship(&self, other_id: u32) -> Relationship {
        *self
            .relationships
            .get(&other_id)
            .unwrap_or(&Relationship::Neutral)
    }

    /// 通行可能かどうかを確認
    pub fn can_pass_through(&self, other_id: u32) -> bool {
        self.get_relationship(other_id).allows_passage()
    }

    /// 攻撃可能かどうかを確認
    pub fn can_attack(&self, other_id: u32) -> bool {
        self.get_relationship(other_id).allows_attack()
    }

    /// ゴールドを追加
    pub fn add_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    /// ゴールドを支払う
    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// 外交ポイントを追加
    pub fn add_diplomatic_points(&mut self, amount: u32) {
        self.diplomatic_points += amount;
    }

    /// 外交アクションに必要なコストを計算
    pub fn diplomatic_action_cost(&self, other_id: u32, base_cost: u32) -> u32 {
        let relationship = self.get_relationship(other_id);
        (base_cost as f32 * relationship.cost_modifier()) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_creation() {
        let faction = Faction::new(
            1,
            "テスト勢力".to_string(),
            FactionType::Player,
            (255, 0, 0), // 赤色
        );

        assert_eq!(faction.id, 1);
        assert_eq!(faction.name, "テスト勢力");
        assert_eq!(faction.faction_type, FactionType::Player);
        assert_eq!(faction.color, (255, 0, 0));
        assert_eq!(faction.gold, 100);
        assert_eq!(faction.diplomatic_points, 0);
        assert!(faction.relationships.is_empty());
    }

    #[test]
    fn test_relationship_management() {
        let mut faction1 = Faction::new(
            1,
            "プレイヤー勢力".to_string(),
            FactionType::Player,
            (0, 0, 255),
        );

        let faction2_id = 2;

        // 初期状態は中立
        assert_eq!(
            faction1.get_relationship(faction2_id),
            Relationship::Neutral
        );

        // 関係を設定
        faction1.set_relationship(faction2_id, Relationship::Allied);
        assert_eq!(faction1.get_relationship(faction2_id), Relationship::Allied);

        // 通行可能性
        assert!(faction1.can_pass_through(faction2_id));

        // 攻撃不可
        assert!(!faction1.can_attack(faction2_id));

        // 関係を敵対に変更
        faction1.set_relationship(faction2_id, Relationship::Hostile);
        assert_eq!(
            faction1.get_relationship(faction2_id),
            Relationship::Hostile
        );

        // 通行不可
        assert!(!faction1.can_pass_through(faction2_id));

        // 攻撃可能
        assert!(faction1.can_attack(faction2_id));
    }

    #[test]
    fn test_gold_management() {
        let mut faction = Faction::new(
            1,
            "テスト勢力".to_string(),
            FactionType::Player,
            (255, 0, 0),
        );

        assert_eq!(faction.gold, 100);

        // ゴールド追加
        faction.add_gold(50);
        assert_eq!(faction.gold, 150);

        // ゴールド消費（成功）
        assert!(faction.spend_gold(100));
        assert_eq!(faction.gold, 50);

        // ゴールド消費（失敗）
        assert!(!faction.spend_gold(100));
        assert_eq!(faction.gold, 50); // 変化なし
    }

    #[test]
    fn test_diplomatic_costs() {
        let mut faction = Faction::new(
            1,
            "プレイヤー勢力".to_string(),
            FactionType::Player,
            (0, 0, 255),
        );

        let ally_id = 2;
        let neutral_id = 3;
        let hostile_id = 4;

        faction.set_relationship(ally_id, Relationship::Allied);
        faction.set_relationship(neutral_id, Relationship::Neutral);
        faction.set_relationship(hostile_id, Relationship::Hostile);

        let base_cost = 100;

        // 同盟相手とは安価
        assert_eq!(faction.diplomatic_action_cost(ally_id, base_cost), 50);

        // 中立勢力とは基本コスト
        assert_eq!(faction.diplomatic_action_cost(neutral_id, base_cost), 100);

        // 敵対勢力とは高価
        assert_eq!(faction.diplomatic_action_cost(hostile_id, base_cost), 150);
    }

    #[test]
    fn test_relationship_properties() {
        // コスト修正の確認
        assert_eq!(Relationship::Friendly.cost_modifier(), 0.8);
        assert_eq!(Relationship::Neutral.cost_modifier(), 1.0);
        assert_eq!(Relationship::AtWar.cost_modifier(), 2.0);

        // 通行可能性の確認
        assert!(Relationship::Allied.allows_passage());
        assert!(Relationship::Neutral.allows_passage());
        assert!(!Relationship::AtWar.allows_passage());

        // 攻撃可能性の確認
        assert!(Relationship::AtWar.allows_attack());
        assert!(Relationship::Hostile.allows_attack());
        assert!(!Relationship::Friendly.allows_attack());
    }
}
