mod events;

use crate::events::EventBus;
use anyhow::Result;
pub use events::GameEvent;
use std::time::Duration;

#[derive(Debug)]
pub struct LoopConfig {
    pub frame_rate: u32,
    pub update_rate: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        LoopConfig {
            frame_rate: 60,
            update_rate: 60,
        }
    }
}

/// ゲームエンジンの主要な構造体
#[derive(Clone)]
pub struct Engine {
    event_bus: EventBus,
    running: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    /// イベントバスへの参照を取得
    pub fn event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    /// イベントの購読を登録
    pub fn subscribe(&self, event_type: &str) -> Result<crossbeam_channel::Receiver<GameEvent>> {
        self.event_bus.subscribe(event_type)
    }

    /// イベントを発行
    pub fn publish(&self, event_type: &str, event: GameEvent) -> Result<()> {
        self.event_bus.publish(event_type, event)
    }

    /// エンジンの実行を開始
    pub fn run(&mut self) -> Result<()> {
        self.start()?;
        Ok(())
    }

    /// ゲームループを開始
    pub fn start(&mut self) -> Result<()> {
        self.running = true;
        self.publish("engine", GameEvent::Start)?;
        Ok(())
    }

    /// ゲームループを停止
    pub fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.publish("engine", GameEvent::Stop)?;
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            event_bus: EventBus::new(),
            running: false,
        }
    }
}

pub struct GameLoop {
    config: LoopConfig,
    receiver: crossbeam_channel::Receiver<GameEvent>,
}

impl GameLoop {
    pub fn new(config: LoopConfig, receiver: crossbeam_channel::Receiver<GameEvent>) -> Self {
        GameLoop { config, receiver }
    }

    pub fn run(&mut self) -> Result<()> {
        let frame_duration = Duration::from_secs_f32(1.0 / self.config.frame_rate as f32);

        loop {
            match self.receiver.try_recv() {
                Ok(GameEvent::Stop) => break,
                Ok(GameEvent::Update { delta }) => {
                    log::debug!("Update: delta = {}", delta);
                }
                Ok(event) => log::debug!("Received event: {:?}", event),
                Err(crossbeam_channel::TryRecvError::Empty) => (),
                Err(e) => log::error!("Error receiving event: {}", e),
            }

            std::thread::sleep(frame_duration);
        }

        Ok(())
    }
}
