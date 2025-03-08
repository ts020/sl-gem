use anyhow::Result;
use engine::{Engine, GameEvent, LoopConfig};
use log::{info, LevelFilter};
use std::{thread, time::Duration};

fn main() -> Result<()> {
    // ロガーの初期化
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    println!("CLIゲームへようこそ！");
    info!("ゲームを起動します...");

    // エンジンの初期化
    let mut engine = Engine::new();
    let event_bus = engine.event_bus();

    // システムイベントの購読
    let receiver = engine.subscribe("system")?;

    // ゲームループの設定
    let config = LoopConfig::default();
    let mut game_loop = engine::GameLoop::new(config, receiver);

    // エンジンの起動
    engine.run()?;

    // 別スレッドでテスト用のUpdateイベントを送信
    let event_bus_clone = event_bus.clone();
    thread::spawn(move || {
        // 3秒間、0.5秒ごとにUpdateイベントを送信
        for i in 0..6 {
            thread::sleep(Duration::from_millis(500));
            event_bus_clone
                .publish("system", GameEvent::Update { delta: 0.016 })
                .unwrap();
            if i == 5 {
                // 最後にStopイベントを送信
                event_bus_clone.publish("system", GameEvent::Stop).unwrap();
            }
        }
    });

    // ゲームループの実行
    match game_loop.run() {
        Ok(_) => info!("ゲームループが正常に終了しました。"),
        Err(e) => log::error!("ゲームループでエラーが発生しました: {}", e),
    }

    println!("ゲーム終了。");
    Ok(())
}
