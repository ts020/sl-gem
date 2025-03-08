use anyhow::Result;
use engine::gui::map_gui::{MapGUI, MapViewOptions};
use engine::{Engine, GameEvent, LoopConfig};
use log::{info, LevelFilter};
use model::{Cell, CellType, Faction, FactionType, Map, Position, Unit, UnitType};
use rand::{thread_rng, Rng};
use std::{thread, time::Duration};

/// サンプルマップを作成
fn create_demo_map() -> Map {
    let width = 20;
    let height = 15;
    let mut map = Map::new(width, height);
    let mut rng = thread_rng();

    // ランダムなマップを生成
    for x in 0..width as i32 {
        for y in 0..height as i32 {
            let position = Position::new(x, y);
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
            Position::new(rng.gen_range(0..5), rng.gen_range(0..5)),
        ));
    }

    // 同盟勢力のユニット
    for i in 0..3 {
        units.push(Unit::new(
            i + 6,
            format!("同盟ユニット{}", i + 1),
            UnitType::Infantry,
            2, // 同盟勢力ID
            Position::new(rng.gen_range(5..10), rng.gen_range(0..5)),
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
            Position::new(rng.gen_range(10..15), rng.gen_range(5..10)),
        ));
    }

    units
}

/// マップの状態をコンソールに表示
fn print_map_info(map_gui: &MapGUI) {
    if let Some(map) = map_gui.get_map() {
        println!("マップサイズ: {}x{}", map.width, map.height);

        let selected_pos = map_gui.get_selected_position();
        if let Some(pos) = selected_pos {
            // Positionは外部型なのでデバッグ形式で表示
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
}

fn main() -> Result<()> {
    // ロガーの初期化
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_timestamp_millis()
        .init();

    println!("SL-GEMゲームへようこそ！");
    println!("戦略エンジン実装テスト\n");
    info!("ゲームを起動します...");

    // エンジンの初期化
    let mut engine = Engine::new();
    let event_bus = engine.event_bus();

    // システムイベントの購読
    let receiver = engine.subscribe("system")?;

    // MapGUIの初期化
    let mut map_gui = MapGUI::new(event_bus.clone());
    info!("MapGUIを初期化しました");

    // サンプルマップとユニットを設定
    let map = create_demo_map();
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
    };
    map_gui.set_view_options(view_options);

    // ゲームループの設定
    let config = LoopConfig::default();
    let mut game_loop = engine::GameLoop::new(config, receiver);

    // エンジンの起動
    engine.run()?;

    // マップの情報を表示
    println!("\n初期マップ情報:");
    print_map_info(&map_gui);

    // テスト用の動作を直接実行
    thread::sleep(Duration::from_millis(500));

    // マップのある位置を選択
    println!("\n位置(5, 5)を選択します...");
    let pos = Position::new(5, 5);
    if let Err(e) = map_gui.select_position(pos) {
        println!("位置選択でエラー: {}", e);
    }

    thread::sleep(Duration::from_millis(1000));
    println!("\n選択後のマップ情報:");
    print_map_info(&map_gui);

    // マップをスクロール
    println!("\nマップをスクロールします...");
    map_gui.scroll(100, 50);

    thread::sleep(Duration::from_millis(1000));
    println!("\nスクロール後のマップ情報:");
    print_map_info(&map_gui);

    // マップをズーム
    println!("\nマップをズームします...");
    map_gui.zoom(1.5);

    thread::sleep(Duration::from_millis(1000));
    println!("\nズーム後のマップ情報:");
    print_map_info(&map_gui);

    // 別スレッドでStopイベントを送信（5秒後）
    let event_bus_clone = event_bus.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        event_bus_clone.publish("system", GameEvent::Stop).unwrap();
    });

    // ゲームループの実行
    match game_loop.run() {
        Ok(_) => info!("ゲームループが正常に終了しました。"),
        Err(e) => log::error!("ゲームループでエラーが発生しました: {}", e),
    }

    println!("\nゲーム終了。");
    Ok(())
}
