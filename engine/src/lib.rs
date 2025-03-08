pub mod core;
pub mod events;
pub mod gui;

use self::core::{GameLoop as CoreGameLoop, LoopConfig as CoreLoopConfig};
pub use self::events::{EventBus, GameEvent, LogLevel, PrioritizedEvent, Priority};
pub use self::gui::{map_gui::MapGUI, map_gui::MapViewOptions};
// modelのPositionをre-exportしない - 直接modelからインポートする
use anyhow::Result;

// CoreLoopConfigをLoopConfigとして再エクスポート
pub type LoopConfig = CoreLoopConfig;

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
        let prioritized_receiver = self.event_bus.subscribe(event_type)?;

        // PrioritizedEventからGameEventに変換するチャネルを作成
        let (sender, receiver) = crossbeam_channel::bounded(100);

        // 別スレッドでPrioritizedEventを受信してGameEventに変換して送信
        std::thread::spawn(move || {
            while let Ok(prioritized_event) = prioritized_receiver.recv() {
                if sender.send(prioritized_event.event).is_err() {
                    break;
                }
            }
        });

        Ok(receiver)
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

// GameEventを受け取るシンプルなGameLoopのラッパー
pub struct GameLoop {
    config: LoopConfig,
    receiver: crossbeam_channel::Receiver<GameEvent>,
}

impl GameLoop {
    pub fn new(config: LoopConfig, receiver: crossbeam_channel::Receiver<GameEvent>) -> Self {
        GameLoop { config, receiver }
    }

    pub fn run(&mut self) -> Result<()> {
        // PrioritizedEventチャンネルを作成
        let (sender, prioritized_receiver) = crossbeam_channel::bounded(100);

        // 受信したGameEventをPrioritizedEventに変換して送信するスレッド
        let receiver = self.receiver.clone();
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                let priority = event.default_priority();
                if sender
                    .send(PrioritizedEvent {
                        priority,
                        event: event.clone(),
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        // コアGameLoopを初期化して実行
        let mut core_loop = CoreGameLoop::new(self.config.clone(), prioritized_receiver);
        core_loop.run()
    }
}
