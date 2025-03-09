//! グラフィカルマップGUI拡張
//!
//! MapGUIにグラフィカルレンダリング機能を追加します。

use anyhow::Result;
use std::sync::{Arc, Mutex};
use winit::window::Window;

use crate::events::EventBus;
use crate::graphics::renderer::map_renderer::MapRenderer;
use crate::gui::map_gui::MapGUI;

/// レンダリングモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// ASCII表示
    Ascii,
    /// グラフィカル表示
    Graphical,
}

/// グラフィカルマップGUI
///
/// MapGUIにグラフィカルレンダリング機能を追加した拡張版です。
pub struct GraphicalMapGUI {
    /// 内部のMapGUIインスタンス
    pub map_gui: MapGUI,
    /// レンダリングモード
    pub render_mode: RenderMode,
    /// マップレンダラー
    pub map_renderer: Option<Arc<Mutex<MapRenderer>>>,
}

impl GraphicalMapGUI {
    /// 新しいGraphicalMapGUIインスタンスを作成
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            map_gui: MapGUI::new(event_bus),
            render_mode: RenderMode::Ascii,
            map_renderer: None,
        }
    }

    /// 既存のMapGUIからGraphicalMapGUIを作成
    pub fn from_map_gui(map_gui: MapGUI) -> Self {
        Self {
            map_gui,
            render_mode: RenderMode::Ascii,
            map_renderer: None,
        }
    }

    /// グラフィカルレンダリングを有効化
    pub async fn enable_graphical_rendering(&mut self, window: &Window) -> Result<()> {
        if self.map_renderer.is_none() {
            // マップレンダラーを初期化
            let mut renderer = MapRenderer::new(window).await?;

            // アセットを読み込む
            // game/assetsディレクトリからアセットを読み込む
            let tileset_path = "game/assets/textures/tiles/default_tileset.png";
            let unitset_path = "game/assets/textures/units/default_unitset.png";

            // アセットが存在するか確認
            if !std::path::Path::new(tileset_path).exists() {
                println!(
                    "警告: タイルセットファイルが見つかりません: {}",
                    tileset_path
                );
                println!("デフォルトのタイルセットを使用します。");
                // 実際のアセットが用意されるまでは、ダミーのテクスチャを使用
            }

            if !std::path::Path::new(unitset_path).exists() {
                println!(
                    "警告: ユニットセットファイルが見つかりません: {}",
                    unitset_path
                );
                println!("デフォルトのユニットセットを使用します。");
                // 実際のアセットが用意されるまでは、ダミーのテクスチャを使用
            }

            // アセットを読み込む
            println!("アセットの読み込みを試みます...");

            // アセットの読み込みを試みるが、失敗してもエラーを返さない
            let asset_result = renderer.load_assets(tileset_path, unitset_path);
            match asset_result {
                Ok(_) => println!("アセットを正常に読み込みました"),
                Err(e) => {
                    println!(
                        "アセットの読み込みに失敗しましたが、処理を続行します: {}",
                        e
                    );
                    println!("シェーダーでインスタンスの色を直接使用するため、テクスチャがなくても色分けされます");
                }
            }

            self.map_renderer = Some(Arc::new(Mutex::new(renderer)));
        }

        self.render_mode = RenderMode::Graphical;
        // マップ更新イベントを発行
        self.map_gui.event_bus.publish(
            "map_gui",
            crate::events::GameEvent::Log {
                message: "マップ表示が更新されました".to_string(),
                level: crate::events::LogLevel::Info,
            },
        )?;

        Ok(())
    }

    /// グラフィカルレンダリングを無効化
    pub fn disable_graphical_rendering(&mut self) -> Result<()> {
        self.render_mode = RenderMode::Ascii;
        // マップ更新イベントを発行
        self.map_gui.event_bus.publish(
            "map_gui",
            crate::events::GameEvent::Log {
                message: "マップ表示が更新されました".to_string(),
                level: crate::events::LogLevel::Info,
            },
        )?;

        Ok(())
    }

    /// レンダリングモードを切り替え
    pub async fn toggle_render_mode(&mut self, window: &Window) -> Result<()> {
        match self.render_mode {
            RenderMode::Ascii => self.enable_graphical_rendering(window).await?,
            RenderMode::Graphical => self.disable_graphical_rendering()?,
        }

        Ok(())
    }

    /// マップをレンダリング
    pub fn render(&self) -> Result<()> {
        match self.render_mode {
            RenderMode::Ascii => {
                // ASCII表示
                self.map_gui.print_ascii_map();
                Ok(())
            }
            RenderMode::Graphical => {
                // グラフィカル表示
                if let Some(renderer) = &self.map_renderer {
                    if let Some(map) = self.map_gui.map.as_ref() {
                        let mut renderer = renderer.lock().unwrap();
                        renderer.render(map, &self.map_gui.units, &self.map_gui.view_options)?;
                    }
                }
                Ok(())
            }
        }
    }

    /// ウィンドウサイズが変更されたときの処理
    pub fn handle_resize(&self, width: u32, height: u32) {
        if let Some(renderer) = &self.map_renderer {
            if let Ok(mut renderer) = renderer.lock() {
                renderer.update_viewport(width, height);
            }
        }
    }

    /// 入力イベントを処理
    pub fn handle_input(&self, event: &winit::event::WindowEvent) -> bool {
        if let Some(renderer) = &self.map_renderer {
            if let Ok(mut renderer) = renderer.lock() {
                return renderer.handle_input(event);
            }
        }
        false
    }

    // MapGUIのメソッドを委譲

    /// マップを設定
    pub fn set_map(&mut self, map: model::Map) {
        self.map_gui.set_map(map);
    }

    /// マップを取得
    pub fn get_map(&self) -> Option<&model::Map> {
        self.map_gui.get_map()
    }

    /// ユニットを追加
    pub fn add_unit(&mut self, unit: model::Unit) {
        self.map_gui.add_unit(unit);
    }

    /// ユニットを更新
    pub fn update_unit(&mut self, unit: model::Unit) -> bool {
        self.map_gui.update_unit(unit)
    }

    /// ユニットを削除
    pub fn remove_unit(&mut self, unit_id: u32) -> bool {
        self.map_gui.remove_unit(unit_id)
    }

    /// IDでユニットを取得
    pub fn get_unit(&self, unit_id: u32) -> Option<&model::Unit> {
        self.map_gui.get_unit(unit_id)
    }

    /// 指定された位置にあるユニットを取得
    pub fn get_unit_at_position(&self, position: &model::MapPosition) -> Option<&model::Unit> {
        self.map_gui.get_unit_at_position(position)
    }

    /// 表示オプションを設定
    pub fn set_view_options(&mut self, options: crate::gui::map_gui::MapViewOptions) {
        self.map_gui.set_view_options(options);
    }

    /// 表示オプションを取得
    pub fn get_view_options(&self) -> &crate::gui::map_gui::MapViewOptions {
        self.map_gui.get_view_options()
    }

    /// マップをスクロール
    pub fn scroll(&mut self, dx: i32, dy: i32) {
        self.map_gui.scroll(dx, dy);
    }

    /// マップのズームを変更
    pub fn zoom(&mut self, factor: f32) {
        self.map_gui.zoom(factor);
    }

    /// セルを選択
    pub fn select_position(&mut self, position: model::MapPosition) -> Result<()> {
        self.map_gui.select_position(position)
    }

    /// 選択位置を取得
    pub fn get_selected_position(&self) -> Option<model::MapPosition> {
        self.map_gui.get_selected_position()
    }

    /// 選択ユニットを取得
    pub fn get_selected_unit(&self) -> Option<&model::Unit> {
        self.map_gui.get_selected_unit()
    }

    /// 選択解除
    pub fn clear_selection(&mut self) {
        self.map_gui.clear_selection();
    }

    /// 特定の位置をハイライト表示
    pub fn highlight_positions(&mut self, positions: Vec<model::MapPosition>) {
        self.map_gui.highlight_positions(positions);
    }

    /// 現在ハイライト表示されている位置を取得
    pub fn get_highlight_positions(&self) -> &[model::MapPosition] {
        self.map_gui.get_highlight_positions()
    }

    /// スクリーン座標からマップ座標への変換
    pub fn screen_to_map_position(&self, screen_x: i32, screen_y: i32) -> model::MapPosition {
        self.map_gui.screen_to_map_position(screen_x, screen_y)
    }

    /// マップ座標からスクリーン座標への変換
    pub fn map_to_screen_position(&self, map_x: i32, map_y: i32) -> (i32, i32) {
        self.map_gui.map_to_screen_position(map_x, map_y)
    }
}
