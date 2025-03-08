use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// ゲーム内で発生する様々なイベントを表現する列挙型
#[derive(Clone, Debug)]
pub enum GameEvent {
    Start,
    Stop,
    Update { delta: f32 },
    // 後で必要なイベントを追加
}

/// イベントバスの実装
#[derive(Clone)]
pub struct EventBus {
    senders: Arc<Mutex<HashMap<String, Vec<Sender<GameEvent>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// 特定のイベントタイプの購読を登録
    pub fn subscribe(&self, event_type: &str) -> anyhow::Result<Receiver<GameEvent>> {
        let (sender, receiver) = bounded(100);
        let mut senders = self.senders.lock().unwrap();
        senders
            .entry(event_type.to_string())
            .or_default()
            .push(sender);
        Ok(receiver)
    }

    /// イベントを発行
    pub fn publish(&self, event_type: &str, event: GameEvent) -> anyhow::Result<()> {
        let senders = self.senders.lock().unwrap();
        if let Some(event_senders) = senders.get(event_type) {
            for sender in event_senders {
                sender.send(event.clone())?;
            }
        }
        Ok(())
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
            match received_event {
                GameEvent::Start => Ok(()),
                _ => panic!("Unexpected event received"),
            }
        } else {
            panic!("No event received");
        }
    }
}
