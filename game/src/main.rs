use anyhow::Result;
use engine::gui::{graphical_map_gui::GraphicalMapGUI, map_gui::MapViewOptions, RenderMode};
use engine::{Engine, GameEvent, LoopConfig};
use log::{info, LevelFilter};
use model::{Cell, CellType, Faction, FactionType, Map, MapPosition, Unit, UnitType};
use rand::{thread_rng, Rng};
use std::{thread, time::Duration};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// サンプルマップを作成
fn create_demo_map() -> Map {
    let width = 20;
    let height = 15;
    let mut map = Map::new(width, height);
    let mut rng = thread_rng();

    // ランダムなマップを生成
    for x in 0..width as i32 {
        for y in 0..height as i32 {
            let position = MapPosition::new(x, y);
            let cell_type = match rng.gen_range(0..100) {
                0..=60 => CellType::Plain,
                61..=75 => CellType::Forest,
                76..=85 => CellType::Mountain,
                86..=90 => CellType::Road,
                91..=95 => CellType::Water,
                _ => CellType::City,
            };

            let cell = if cell_type == CellType::City {
                // 都市は20%の確率で勢力に所属
                if rng.gen_range(0..100) < 20 {
                    Cell::with_faction(cell_type, rng.gen_range(1..=3))
                } else {
                    Cell::new(cell_type)
                }
            } else {
                Cell::new(cell_type)
            };

            map.set_cell(position, cell);
        }
    }

    map
}

/// サンプル勢力を作成
#[allow(dead_code)]
fn create_demo_factions() -> Vec<Faction> {
    vec![
        Faction::new(
            1,
            "プレイヤー勢力".to_string(),
            FactionType::Player,
            (0, 0, 255),
        ),
        Faction::new(2, "同盟勢力".to_string(), FactionType::Ally, (0, 255, 0)),
        Faction::new(3, "敵対勢力".to_string(), FactionType::Rival, (255, 0, 0)),
    ]
}

/// サンプルユニットを作成
fn create_demo_units() -> Vec<Unit> {
    let mut units = Vec::new();
    let mut rng = thread_rng();

    // プレイヤー勢力のユニット
    for i in 0..5 {
        let unit_type = match i % 3 {
            0 => UnitType::Infantry,
            1 => UnitType::Cavalry,
            _ => UnitType::Ranged,
        };

        units.push(Unit::new(
            i + 1,
            format!("プレイヤーユニット{}", i + 1),
            unit_type,
            1, // プレイヤー勢力ID
            MapPosition::new(rng.gen_range(0..5), rng.gen_range(0..5)),
        ));
    }

    // 同盟勢力のユニット
    for i in 0..3 {
        units.push(Unit::new(
            i + 6,
            format!("同盟ユニット{}", i + 1),
            UnitType::Infantry,
            2, // 同盟勢力ID
            MapPosition::new(rng.gen_range(5..10), rng.gen_range(0..5)),
        ));
    }

    // 敵対勢力のユニット
    for i in 0..4 {
        let unit_type = match i % 2 {
            0 => UnitType::Infantry,
            _ => UnitType::Ranged,
        };

        units.push(Unit::new(
            i + 9,
            format!("敵対ユニット{}", i + 1),
            unit_type,
            3, // 敵対勢力ID
            MapPosition::new(rng.gen_range(10..15), rng.gen_range(5..10)),
        ));
    }

    units
}

/// マップの状態をコンソールに表示（固定位置に表示）
fn print_map_info(engine: &Engine, map_gui: &GraphicalMapGUI) {
    // グラフィカルモードの場合は表示しない
    if map_gui.render_mode == RenderMode::Graphical {
        return;
    }

    // ANSIエスケープシーケンスを使用して画面をクリアし、カーソルを先頭に移動
    print!("\x1B[2J\x1B[H");

    // マップの文字列を取得
    let map_string = engine.print_map_ascii(&map_gui.map_gui);

    // マップの表示
    println!("ASCIIマップ表示:");
    println!("{}", map_string);

    // 固定位置に凡例と詳細情報を表示
    println!("\n凡例:");
    println!("地形: .=平地, T=森, ^=山, ~=水域, ==道路, C=都市, B=拠点");
    println!("ユニット: 1=プレイヤー勢力, 2=同盟勢力, 3=敵対勢力");
    println!("状態: [x]=選択中, *x*=ハイライト表示\n");

    // マップの詳細情報
    if let Some(map) = map_gui.get_map() {
        println!("マップサイズ: {}x{}", map.width, map.height);

        let selected_pos = map_gui.get_selected_position();
        if let Some(pos) = selected_pos {
            // MapPositionは外部型なのでデバッグ形式で表示
            println!("選択中の位置: {:?}", pos);

            if let Some(cell) = map.get_cell(&pos) {
                println!("  セルタイプ: {:?}", cell.cell_type);
                println!("  所有勢力: {:?}", cell.faction_id);
            }

            if let Some(unit) = map_gui.get_unit_at_position(&pos) {
                println!("  ユニット: {} (ID: {})", unit.name, unit.id);
                println!("    タイプ: {:?}", unit.unit_type);
                println!("    勢力ID: {}", unit.faction_id);
                println!("    体力: {}", unit.health);
            }
        }

        if let Some(unit) = map_gui.get_selected_unit() {
            println!("選択中のユニット: {} (ID: {})", unit.name, unit.id);
            println!("  位置: {:?}", unit.position);
            println!("  攻撃力: {}", unit.attack_power());
            println!("  防御力: {}", unit.defense_power());
        }
    }

    let view_options = map_gui.get_view_options();
    println!("表示設定:");
    println!("  ズーム: {:.1}", view_options.zoom);
    println!(
        "  スクロール: ({}, {})",
        view_options.scroll_x, view_options.scroll_y
    );

    // 標準出力をフラッシュして即座に表示を反映
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロガーの初期化
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    println!("SL-GEMゲームへようこそ！");
    println!("戦略エンジン実装テスト\n");
    info!("ゲームを起動します...");

    // ウィンドウとイベントループを作成
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_title("SL-GEM グラフィカルマップデモ");
    window.set_inner_size(winit::dpi::PhysicalSize::new(800, 600));

    // エンジンの初期化
    let mut engine = Engine::new();
    let event_bus = engine.event_bus();

    // システムイベントの購読
    let receiver = engine.subscribe("system")?;

    // GraphicalMapGUIの初期化
    let mut map_gui = GraphicalMapGUI::new(event_bus.clone());
    info!("GraphicalMapGUIを初期化しました");

    // サンプルマップとユニットを設定
    let map = create_demo_map();

    // マップの統計情報を表示（デバッグ用）
    let mut plain_count = 0;
    let mut forest_count = 0;
    let mut mountain_count = 0;
    let mut water_count = 0;
    let mut road_count = 0;
    let mut city_count = 0;
    let mut base_count = 0;

    for x in 0..map.width as i32 {
        for y in 0..map.height as i32 {
            let pos = MapPosition::new(x, y);
            if let Some(cell) = map.get_cell(&pos) {
                match cell.cell_type {
                    CellType::Plain => plain_count += 1,
                    CellType::Forest => forest_count += 1,
                    CellType::Mountain => mountain_count += 1,
                    CellType::Water => water_count += 1,
                    CellType::Road => road_count += 1,
                    CellType::City => city_count += 1,
                    CellType::Base => base_count += 1,
                }

                // パリティチェックのデバッグ出力
                let parity = (x + y) % 2;
                if x < 10 && y < 10 {
                    // 左上の一部だけ出力
                    println!("位置 ({}, {}), パリティ: {}", x, y, parity);
                }
            }
        }
    }

    println!("マップの統計情報:");
    println!("平地: {} マス", plain_count);
    println!("森林: {} マス", forest_count);
    println!("山地: {} マス", mountain_count);
    println!("水域: {} マス", water_count);
    println!("道路: {} マス", road_count);
    println!("都市: {} マス", city_count);
    println!("拠点: {} マス", base_count);

    map_gui.set_map(map);
    info!("サンプルマップを生成しました");

    let units = create_demo_units();
    for unit in units {
        map_gui.add_unit(unit);
    }
    info!("サンプルユニットを配置しました");

    // マップの表示設定を調整
    let view_options = MapViewOptions {
        tile_size: 32,
        scroll_x: 0,
        scroll_y: 0,
        zoom: 1.2,
        show_grid: true,
        viewport_width: 20,
        viewport_height: 15,
    };
    map_gui.set_view_options(view_options);

    // グラフィカルレンダリングを有効化
    info!("グラフィカルレンダリングを有効化します...");
    match map_gui.enable_graphical_rendering(&window).await {
        Ok(_) => info!("グラフィカルレンダリングを有効化しました"),
        Err(e) => {
            log::error!("グラフィカルレンダリングの有効化に失敗しました: {}", e);
            println!("グラフィカルレンダリングの有効化に失敗しました。ASCIIモードで続行します。");
        }
    }

    // ゲームループの設定
    let config = LoopConfig::default();
    let mut game_loop = engine::GameLoop::new(config, receiver);

    // エンジンの起動
    engine.run()?;

    // マップのある位置を選択
    let pos = MapPosition::new(5, 5);
    if let Err(e) = map_gui.select_position(pos) {
        println!("位置選択でエラー: {}", e);
    } else {
        // 選択した位置の周囲をハイライト表示（移動可能範囲のシミュレーション）
        let highlights = vec![
            pos.moved(1, 0),
            pos.moved(-1, 0),
            pos.moved(0, 1),
            pos.moved(0, -1),
            pos.moved(1, 1),
            pos.moved(-1, -1),
            pos.moved(1, -1),
            pos.moved(-1, 1),
        ];
        map_gui.highlight_positions(highlights);
    }

    // ASCIIモードの場合のみ、コンソールに情報を表示
    if map_gui.render_mode == RenderMode::Ascii {
        // 初期マップ情報を表示
        print_map_info(&engine, &map_gui);
        println!("自動スクロールデモを開始します。1秒後に移動を開始します...");
        thread::sleep(Duration::from_secs(1));

        // 選択状態を表示
        print_map_info(&engine, &map_gui);
        println!("位置(5, 5)を選択しました。1秒後に自動スクロールを開始します...");
        thread::sleep(Duration::from_secs(1));

        // 自動スクロールデモ: 縦に5回、横に2回、上に3回
        println!("自動スクロールを開始します...");

        // 縦に5回スクロール（下方向）
        for i in 1..=5 {
            map_gui.scroll(0, 30);
            print_map_info(&engine, &map_gui);
            println!("縦方向スクロール {}/5", i);
            thread::sleep(Duration::from_secs(1));
        }

        // 横に2回スクロール（右方向）
        for i in 1..=2 {
            map_gui.scroll(30, 0);
            print_map_info(&engine, &map_gui);
            println!("横方向スクロール {}/2", i);
            thread::sleep(Duration::from_secs(1));
        }

        // 上に3回スクロール（上方向）
        for i in 1..=3 {
            map_gui.scroll(0, -30);
            print_map_info(&engine, &map_gui);
            println!("上方向スクロール {}/3", i);
            thread::sleep(Duration::from_secs(1));
        }

        // ズームしてみる
        map_gui.zoom(1.5);
        print_map_info(&engine, &map_gui);
        println!("マップをズームしました。デモを終了します...");
        thread::sleep(Duration::from_secs(1));
    } else {
        // グラフィカルモードの場合は、イベントループを実行
        println!("グラフィカルモードでマップを表示しています。");
        println!("矢印キーでスクロール、+/-キーでズームができます。");
        println!("ESCキーで終了します。");

        // 別スレッドでStopイベントを送信（30秒後）
        let event_bus_clone = event_bus.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(30));
            event_bus_clone.publish("system", GameEvent::Stop).unwrap();
        });

        // イベントループを実行
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    // エンジンを停止
                    event_bus.publish("system", GameEvent::Stop).unwrap();
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(keycode),
                                    ..
                                },
                            ..
                        },
                    ..
                } => match keycode {
                    VirtualKeyCode::Escape => {
                        *control_flow = ControlFlow::Exit;
                        // エンジンを停止
                        event_bus.publish("system", GameEvent::Stop).unwrap();
                    }
                    VirtualKeyCode::Up => {
                        map_gui.scroll(0, -10);
                    }
                    VirtualKeyCode::Down => {
                        map_gui.scroll(0, 10);
                    }
                    VirtualKeyCode::Left => {
                        map_gui.scroll(-10, 0);
                    }
                    VirtualKeyCode::Right => {
                        map_gui.scroll(10, 0);
                    }
                    VirtualKeyCode::Equals => {
                        map_gui.zoom(1.1);
                    }
                    VirtualKeyCode::Minus => {
                        map_gui.zoom(0.9);
                    }
                    _ => {}
                },
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    map_gui.handle_resize(new_size.width, new_size.height);
                }
                Event::MainEventsCleared => {
                    // レンダリング
                    map_gui.render().ok();

                    // 次のフレームを要求
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }

    // ゲームループの実行
    match game_loop.run() {
        Ok(_) => info!("ゲームループが正常に終了しました。"),
        Err(e) => log::error!("ゲームループでエラーが発生しました: {}", e),
    }

    println!("\nゲーム終了。");
    Ok(())
}
