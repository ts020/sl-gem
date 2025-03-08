pub mod faction;
pub mod map;
pub mod unit;

pub use crate::faction::{Faction, FactionType, Relationship};
pub use crate::map::{Cell, CellType, Map, Position};
pub use crate::unit::{Unit, UnitStatus, UnitType};

pub fn greet() {
    println!("Model library loaded.");
}
