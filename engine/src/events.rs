use anyhow::Result;
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
        EventBus {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 特定のイベントタイプの購読を登録
    pub fn subscribe(&self, event_type: &str) -> Result<Receiver<GameEvent>> {
        let (sender, receiver) = bounded(100);
        let mut senders = self.senders.lock().unwrap();
        senders
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(sender);
        Ok(receiver)
    }

    /// イベントを発行
    pub fn publish(&self, event_type: &str, event: GameEvent) -> Result<()> {
        let senders = self.senders.lock().unwrap();
        if let Some(sender_list) = senders.get(event_type) {
            for sender in sender_list {
                sender.send(event.clone())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus() {
        let bus = EventBus::new();

        // イベントの購読
        let receiver = bus.subscribe("update").unwrap();

        // イベントの発行
        bus.publish("update", GameEvent::Update { delta: 0.16 })
            .unwrap();

        // 受信したイベントの確認
        if let Ok(GameEvent::Update { delta }) = receiver.recv() {
            assert_eq!(delta, 0.16);
        } else {
            panic!("Expected Update event");
        }
    }
}
