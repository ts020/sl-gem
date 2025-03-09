//! ウィンドウ管理モジュール
//!
//! ウィンドウの作成と管理を担当します。

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as WinitWindow, WindowBuilder},
};

/// ウィンドウ
///
/// ゲームウィンドウの作成と管理を担当します。
pub struct Window {
    window: WinitWindow,
    event_loop: Option<EventLoop<()>>,
}

impl Window {
    /// 新しいウィンドウを作成
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .build(&event_loop)?;

        Ok(Self {
            window,
            event_loop: Some(event_loop),
        })
    }

    /// ウィンドウの内部サイズを取得
    pub fn inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    /// ウィンドウの参照を取得
    pub fn window(&self) -> &WinitWindow {
        &self.window
    }

    /// イベントループを実行
    ///
    /// この関数は、イベントループが終了するまで戻りません。
    pub fn run<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&WinitWindow, &Event<()>) -> ControlFlow + 'static,
    {
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| anyhow::anyhow!("イベントループが既に消費されています"))?;

        event_loop.run(move |event, _, control_flow| {
            // デフォルトではイベントループを継続
            *control_flow = ControlFlow::Poll;

            // コールバック関数を呼び出し
            let result = callback(&self.window, &event);
            *control_flow = result;

            // ウィンドウが閉じられたらイベントループを終了
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit;
            }
        });

        // イベントループが終了することはないので、ここには到達しない
        #[allow(unreachable_code)]
        Ok(())
    }
}

/// ウィンドウイベントハンドラ
///
/// ウィンドウイベントを処理するトレイトです。
pub trait WindowEventHandler {
    /// イベント処理
    ///
    /// ウィンドウイベントを処理します。
    ///
    /// 戻り値:
    /// - `true`: イベントが処理された
    /// - `false`: イベントが処理されなかった
    fn handle_event(&mut self, window: &WinitWindow, event: &WindowEvent) -> bool;
}

/// シェーダーテスト用ウィンドウ
///
/// シェーダーテスト環境のためのウィンドウ管理を行います。
pub struct ShaderTestWindow {
    window: WinitWindow,
    event_loop: Option<EventLoop<()>>,
}

impl ShaderTestWindow {
    /// 新しいシェーダーテストウィンドウを作成
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .with_resizable(true)
            .build(&event_loop)?;

        Ok(Self {
            window,
            event_loop: Some(event_loop),
        })
    }

    /// ウィンドウの参照を取得
    pub fn window(&self) -> &WinitWindow {
        &self.window
    }

    /// ウィンドウの内部サイズを取得
    pub fn inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    /// カスタムイベントハンドラでイベントループを実行
    pub fn run<F, S>(mut self, mut state: S, mut callback: F) -> Result<()>
    where
        F: FnMut(&mut S, &WinitWindow, &Event<()>) -> ControlFlow + 'static,
        S: 'static,
    {
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| anyhow::anyhow!("イベントループが既に消費されています"))?;

        event_loop.run(move |event, _, control_flow| {
            // デフォルトではイベントループを継続
            *control_flow = ControlFlow::Poll;

            // コールバック関数を呼び出し
            let result = callback(&mut state, &self.window, &event);
            *control_flow = result;

            // ESCキーが押されたか、ウィンドウが閉じられたらイベントループを終了
            if let Event::WindowEvent { event, .. } = &event {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        use winit::event::ElementState;
                        use winit::event::VirtualKeyCode;

                        if let Some(key) = input.virtual_keycode {
                            if key == VirtualKeyCode::Escape && input.state == ElementState::Pressed
                            {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        // イベントループが終了することはないので、ここには到達しない
        #[allow(unreachable_code)]
        Ok(())
    }

    /// EGUIを使用したシェーダーテスト環境用のイベントループを実行（一時的に無効化）
    pub fn run_with_egui<F, S>(_self: Self, _state: S, _callback: F) -> Result<()>
    where
        F: FnMut(&mut S, &WinitWindow, &Event<()>, &mut egui_winit::State) -> ControlFlow + 'static,
        S: 'static,
    {
        // UI機能は一時的に無効化されています
        Err(anyhow::anyhow!("EGUIサポートは現在無効化されています"))
    }
}
