//! SL-GEMゲームエンジン
//!
//! イベントドリブンなゲームエンジンの実装を提供します。

mod core;
mod events;

// 必要な型をpubで再エクスポート
pub use crate::core::{GameLoop, LoopConfig};
pub use crate::events::{EventBus, GameEvent};

use anyhow::Result;
use log::info;

/// ゲームエンジンの状態を管理する構造体
#[derive(Clone)]
pub struct Engine {
    event_bus: EventBus,
    running: bool,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            event_bus: EventBus::new(),
            running: false,
        }
    }

    /// エンジンの初期化と実行
    pub fn run(&mut self) -> Result<()> {
        self.running = true;
        self.event_bus.publish("system", GameEvent::Start)?;
        info!("Engine started");
        Ok(())
    }

    /// エンジンの停止
    pub fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.event_bus.publish("system", GameEvent::Stop)?;
        info!("Engine stopped");
        Ok(())
    }

    /// イベントバスの取得
    pub fn event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(!engine.running);
    }
}