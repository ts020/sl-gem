use crossbeam_channel::{bounded, Receiver, Sender};
use model::Position;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// イベントの優先度を表現する列挙型
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    #[default]
    Normal,
    Low,
}

use std::fmt;

/// ログレベルを表現する列挙型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warning => write!(f, "Warning"),
            LogLevel::Error => write!(f, "Error"),
        }
    }
}

/// ゲーム内で発生する様々なイベントを表現する列挙型
#[derive(Clone, Debug)]
pub enum GameEvent {
    // システムイベント（High Priority）
    Start,
    Stop,
    Pause,
    Resume,

    // ゲーム状態イベント（Normal Priority）
    Update { delta: f32 },
    TurnStart { faction_id: u32 },
    TurnEnd { faction_id: u32 },
    UnitMove { unit_id: u32, position: Position },

    // 情報イベント（Low Priority）
    Log { message: String, level: LogLevel },
    Stats { metric: String, value: f64 },
}

/// イベントとその優先度をカプセル化する構造体
#[derive(Clone, Debug)]
pub struct PrioritizedEvent {
    pub priority: Priority,
    pub event: GameEvent,
}

impl GameEvent {
    /// イベントのデフォルト優先度を返す
    pub fn default_priority(&self) -> Priority {
        match self {
            GameEvent::Start | GameEvent::Stop | GameEvent::Pause | GameEvent::Resume => {
                Priority::High
            }

            GameEvent::Update { .. }
            | GameEvent::TurnStart { .. }
            | GameEvent::TurnEnd { .. }
            | GameEvent::UnitMove { .. } => Priority::Normal,

            GameEvent::Log { .. } | GameEvent::Stats { .. } => Priority::Low,
        }
    }
}

/// イベントバスの実装
#[derive(Clone)]
pub struct EventBus {
    senders: Arc<Mutex<HashMap<String, Vec<Sender<PrioritizedEvent>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// 特定のイベントタイプの購読を登録
    pub fn subscribe(&self, event_type: &str) -> anyhow::Result<Receiver<PrioritizedEvent>> {
        let (sender, receiver) = bounded(100);
        let mut senders = self.senders.lock().unwrap();
        senders
            .entry(event_type.to_string())
            .or_default()
            .push(sender);
        Ok(receiver)
    }

    /// イベントを発行（デフォルトの優先度を使用）
    pub fn publish(&self, event_type: &str, event: GameEvent) -> anyhow::Result<()> {
        self.publish_with_priority(event_type, event, None)
    }

    /// イベントを指定した優先度で発行
    pub fn publish_with_priority(
        &self,
        event_type: &str,
        event: GameEvent,
        priority: Option<Priority>,
    ) -> anyhow::Result<()> {
        let priority = priority.unwrap_or_else(|| event.default_priority());
        let prioritized_event = PrioritizedEvent { priority, event };

        let senders = self.senders.lock().unwrap();
        if let Some(event_senders) = senders.get(event_type) {
            for sender in event_senders {
                sender.send(prioritized_event.clone())?;
            }
        }
        Ok(())
    }

    /// エラーイベントを発行（常にHigh優先度）
    pub fn publish_error(&self, message: String) -> anyhow::Result<()> {
        self.publish_with_priority(
            "error",
            GameEvent::Log {
                message,
                level: LogLevel::Error,
            },
            Some(Priority::High),
        )
    }
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_and_publish() -> anyhow::Result<()> {
        let event_bus = EventBus::new();
        let receiver = event_bus.subscribe("test")?;

        event_bus.publish("test", GameEvent::Start)?;

        if let Ok(received_event) = receiver.try_recv() {
            match received_event.event {
                GameEvent::Start => {
                    assert_eq!(received_event.priority, Priority::High);
                    Ok(())
                }
                _ => panic!("Unexpected event received"),
            }
        } else {
            panic!("No event received");
        }
    }

    #[test]
    fn test_publish_with_priority() -> anyhow::Result<()> {
        let event_bus = EventBus::new();
        let receiver = event_bus.subscribe("test")?;

        // 通常優先度のイベントを高優先度で送信
        event_bus.publish_with_priority(
            "test",
            GameEvent::Update { delta: 0.016 },
            Some(Priority::High),
        )?;

        if let Ok(received_event) = receiver.try_recv() {
            assert_eq!(received_event.priority, Priority::High);
            match received_event.event {
                GameEvent::Update { delta } => {
                    assert_eq!(delta, 0.016);
                    Ok(())
                }
                _ => panic!("Unexpected event received"),
            }
        } else {
            panic!("No event received");
        }
    }

    #[test]
    fn test_error_event() -> anyhow::Result<()> {
        let event_bus = EventBus::new();
        let receiver = event_bus.subscribe("error")?;

        event_bus.publish_error("Test error".to_string())?;

        if let Ok(received_event) = receiver.try_recv() {
            assert_eq!(received_event.priority, Priority::High);
            match received_event.event {
                GameEvent::Log { message, level } => {
                    assert_eq!(message, "Test error");
                    assert_eq!(level, LogLevel::Error);
                    Ok(())
                }
                _ => panic!("Unexpected event received"),
            }
        } else {
            panic!("No event received");
        }
    }
}
