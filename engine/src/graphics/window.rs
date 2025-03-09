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
        let event_loop = self.event_loop.take().ok_or_else(|| {
            anyhow::anyhow!("イベントループが既に消費されています")
        })?;

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