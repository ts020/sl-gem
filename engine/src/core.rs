use crate::GameEvent;
use anyhow::Result;
use crossbeam_channel::Receiver;
use log::{debug, info};
use std::time::{Duration, Instant};

/// ゲームループの設定
pub struct LoopConfig {
    /// 目標フレームレート（FPS）
    pub target_fps: u32,
    /// 最大更新回数/秒
    pub max_updates: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        LoopConfig {
            target_fps: 60,
            max_updates: 60,
        }
    }
}

/// ゲームループの状態を管理
pub struct GameLoop {
    config: LoopConfig,
    event_receiver: Receiver<GameEvent>,
    last_update: Instant,
    accumulated_time: Duration,
    frame_duration: Duration,
}

impl GameLoop {
    pub fn new(config: LoopConfig, event_receiver: Receiver<GameEvent>) -> Self {
        let frame_duration = Duration::from_secs_f64(1.0 / config.target_fps as f64);
        GameLoop {
            config,
            event_receiver,
            last_update: Instant::now(),
            accumulated_time: Duration::ZERO,
            frame_duration,
        }
    }

    /// ゲームループの実行
    pub fn run(&mut self) -> Result<()> {
        info!("Starting game loop");
        
        while let Ok(event) = self.event_receiver.recv() {
            match event {
                GameEvent::Stop => {
                    info!("Stopping game loop");
                    break;
                }
                _ => self.process_frame()?,
            }
        }

        Ok(())
    }

    /// 1フレームの処理
    fn process_frame(&mut self) -> Result<()> {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_update);
        self.last_update = current_time;

        // 時間の蓄積（最大値を制限して極端な更新を防ぐ）
        self.accumulated_time += frame_time.min(Duration::from_secs(1) / self.config.max_updates);

        // 固定時間ステップでの更新
        while self.accumulated_time >= self.frame_duration {
            self.update()?;
            self.accumulated_time -= self.frame_duration;
        }

        // レンダリング
        self.render()?;

        Ok(())
    }

    /// ゲーム状態の更新
    fn update(&mut self) -> Result<()> {
        // イベントキューから非ブロッキングで処理
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                GameEvent::Update { delta } => {
                    // 更新処理
                    debug!("Update frame with delta: {:.3}ms", delta * 1000.0);
                }
                GameEvent::Stop => {
                    info!("Received stop event - shutting down game loop");
                    return Ok(());
                }
                event => {
                    debug!("Received event: {:?}", event);
                }
            }
        }
        Ok(())
    }

    /// レンダリング
    fn render(&self) -> Result<()> {
        // TODO: 実際のレンダリング処理
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::bounded;
    use std::{thread, time::Duration};

    #[test]
    fn test_game_loop_config() {
        let config = LoopConfig::default();
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.max_updates, 60);
    }

    #[test]
    fn test_game_loop_creation() {
        let config = LoopConfig::default();
        let (sender, receiver) = bounded(100);
        let _loop = GameLoop::new(config, receiver);
        // GameLoopが正しく作成されることを確認
        sender.send(GameEvent::Stop).unwrap();
    }

    #[test]
    fn test_game_loop_stop_event() {
        let config = LoopConfig::default();
        let (sender, receiver) = bounded(100);
        let mut game_loop = GameLoop::new(config, receiver);

        // 別スレッドでStopイベントを送信
        let sender_clone = sender.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            sender_clone.send(GameEvent::Stop).unwrap();
        });

        // ゲームループを実行（Stopイベントで終了するはず）
        assert!(game_loop.run().is_ok());
    }

    #[test]
    fn test_game_loop_update_events() {
        let config = LoopConfig::default();
        let (sender, receiver) = bounded(100);
        let mut game_loop = GameLoop::new(config, receiver);

        // 別スレッドで複数のUpdateイベントを送信
        let sender_clone = sender.clone();
        thread::spawn(move || {
            // 3回のUpdateイベントを送信
            for i in 0..3 {
                sender_clone
                    .send(GameEvent::Update {
                        delta: 0.016 * (i + 1) as f32,
                    })
                    .unwrap();
                thread::sleep(Duration::from_millis(50));
            }
            // 最後にStopイベントを送信
            sender_clone.send(GameEvent::Stop).unwrap();
        });

        // ゲームループを実行
        assert!(game_loop.run().is_ok());
    }

    #[test]
    fn test_game_loop_event_order() {
        let config = LoopConfig::default();
        let (sender, receiver) = bounded(100);
        let mut game_loop = GameLoop::new(config, receiver);

        // イベントを順番に送信
        sender.send(GameEvent::Start).unwrap();
        sender.send(GameEvent::Update { delta: 0.016 }).unwrap();
        sender.send(GameEvent::Update { delta: 0.016 }).unwrap();
        sender.send(GameEvent::Stop).unwrap();

        // ゲームループを実行（すべてのイベントが処理されるはず）
        assert!(game_loop.run().is_ok());
    }
}