//! シェーダーテストUI
//!
//! インタラクティブなシェーダーテスト環境のUIを提供します。

use anyhow::Result;
use egui::{Color32, Pos2, Rect, Stroke, TextEdit, Vec2};
use egui_wgpu::renderer::ScreenDescriptor;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wgpu::SamplerDescriptor;
use winit::{
    event::{Event, WindowEvent},
    window::Window as WinitWindow,
};

use super::{Parameter, ShaderSource, ShaderTestRunner, TestCase, TestEnvironmentConfig};
use crate::texture::TextureGenerator;
use crate::window::ShaderTestWindow;
use crate::Texture;

/// シェーダーテストUI
///
/// インタラクティブなシェーダーテスト環境のUIを管理するモジュールです。
pub struct ShaderTestUI {
    /// テスト環境の設定
    config: TestEnvironmentConfig,
    /// シェーダーテストランナー
    runner: Arc<Mutex<ShaderTestRunner>>,
    /// 現在のテストケース
    current_test: Option<TestCase>,
    /// テストケースのリスト
    test_cases: Vec<TestCase>,
    /// エディタに表示中のシェーダーコード
    shader_code: String,
    /// シェーダーコードの変更フラグ
    shader_modified: bool,
    /// コンパイルエラーメッセージ
    compilation_error: Option<String>,
    /// 最終レンダリング時間
    last_render_time: Instant,
    /// 経過時間
    elapsed_time: f32,
    /// アニメーション再生フラグ
    is_playing: bool,
    /// 現在のパラメータ値
    parameter_values: HashMap<String, f32>,
    /// テストケースのインデックス
    current_test_index: usize,
    /// EGUIコンテキスト
    egui_ctx: egui::Context,
    /// EGUIレンダラー
    egui_renderer: egui_wgpu::Renderer,
    /// 出力テクスチャID（EGUI用）
    output_texture_id: Option<egui::TextureId>,
    /// レンダリング結果を表示するテクスチャ
    display_texture: Option<Texture>,
}

impl ShaderTestUI {
    /// 新しいシェーダーテストUIを作成
    pub fn new(
        config: TestEnvironmentConfig,
        runner: ShaderTestRunner,
        window: &WinitWindow,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // EGUIコンテキスト
        let egui_ctx = egui::Context::default();

        // EGUIレンダラー
        let egui_renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);

        // 初期テストケースのリスト
        let test_cases = super::case::create_builtin_testcases();

        let current_test = if !test_cases.is_empty() {
            Some(test_cases[0].clone())
        } else {
            None
        };

        let mut ui = Self {
            config,
            runner: Arc::new(Mutex::new(runner)),
            current_test,
            test_cases,
            shader_code: String::new(),
            shader_modified: false,
            compilation_error: None,
            last_render_time: Instant::now(),
            elapsed_time: 0.0,
            is_playing: true,
            parameter_values: HashMap::new(),
            current_test_index: 0,
            egui_ctx,
            egui_renderer,
            output_texture_id: None,
            display_texture: None,
        };

        // 初期テストケースをロード
        if ui.current_test.is_some() {
            ui.load_current_test();
        }

        ui
    }

    /// EGUIコンテキストを取得
    pub fn context(&self) -> &egui::Context {
        &self.egui_ctx
    }

    /// テストケースをロード
    fn load_current_test(&mut self) {
        let test = match &self.current_test {
            Some(test) => test.clone(),
            None => return,
        };

        // パラメータ値を初期化
        self.parameter_values.clear();
        for param in &test.data.parameters {
            self.parameter_values
                .insert(param.name.clone(), param.default);
        }

        // シェーダーコードをエディタにロード
        if let ShaderSource::Code(code) = &test.data.shader {
            self.shader_code = code.clone();
        } else {
            // 組み込みシェーダーまたはファイルの場合は空文字列
            self.shader_code = String::new();
        }

        self.shader_modified = false;
        self.compilation_error = None;

        // テストをランナーに設定
        let mut runner = self.runner.lock().unwrap();
        runner.set_test_case(test);

        // 初期レンダリング
        let _ = runner.run();
    }

    /// 新しいテストケースを作成
    fn create_new_test(&mut self) {
        let new_test = TestCase::new("new_test")
            .with_description("新しいテストケース")
            .with_shader("test")
            .with_background_color(0.1, 0.1, 0.1, 1.0);

        self.test_cases.push(new_test.clone());
        self.current_test = Some(new_test);
        self.current_test_index = self.test_cases.len() - 1;

        self.load_current_test();
    }

    /// シェーダーコードを適用
    fn apply_shader_code(&mut self) {
        if !self.shader_modified {
            return;
        }

        let test = match &mut self.current_test {
            Some(test) => test,
            None => return,
        };

        // シェーダーコードを更新
        test.data.shader = ShaderSource::Code(self.shader_code.clone());

        // テストをランナーに設定
        let mut runner = self.runner.lock().unwrap();
        runner.set_test_case(test.clone());

        // 再初期化
        if let Err(err) = runner.initialize_resources() {
            self.compilation_error = Some(format!("コンパイルエラー: {}", err));
            return;
        }

        self.compilation_error = None;
        self.shader_modified = false;
    }

    /// UIの更新と描画
    pub fn update(
        &mut self,
        window: &WinitWindow,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
    ) {
        // 時間を更新
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_render_time).as_secs_f32();
        self.last_render_time = now;

        if self.is_playing {
            self.elapsed_time += delta_time;

            // ランナーの時間を設定
            let mut runner = self.runner.lock().unwrap();
            runner.set_time(self.elapsed_time);
        }

        // EGUIフレームの開始
        let egui_start = Instant::now();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        // ランナーでレンダリング
        if self.current_test.is_some() {
            let mut runner = self.runner.lock().unwrap();
            match runner.run() {
                Ok(_) => {
                    // レンダリング結果を取得
                    if let Ok(output_image) = runner.get_output_image() {
                        // EGUIにテクスチャを登録
                        let width = output_image.width();
                        let height = output_image.height();
                        let pixels = output_image.into_raw();

                        // すでにテクスチャIDがあれば更新、なければ新規作成
                        if let Some(texture_id) = self.output_texture_id {
                            // egui 0.22 + wgpu 0.16 互換性調整
                            let image_delta = egui::epaint::ImageDelta {
                                image: egui::epaint::ImageData::Color(
                                    egui::ColorImage::from_rgba_unmultiplied(
                                        [width as usize, height as usize],
                                        &pixels,
                                    )
                                ),
                                pos: None,
                                options: Default::default(),
                            };
                            
                            self.egui_renderer.update_texture(
                                device,
                                queue,
                                texture_id,
                                &image_delta,
                            );
                        } else {
                            // ネイティブテクスチャを登録
                            let texture_id = self.egui_renderer.register_native_texture(
                                device,
                                runner.get_output_texture_view().unwrap(),
                                wgpu::FilterMode::Linear,
                            );
                            self.output_texture_id = Some(texture_id);
                        }
                    }
                }
                Err(err) => {
                    self.compilation_error = Some(format!("レンダリングエラー: {}", err));
                }
            }
        }

        // UIの描画（egui-winit 0.22）
        let mut egui_state = egui_winit::State::new();
        let raw_input = egui_state.take_egui_input(&self.egui_ctx, window);
        self.egui_ctx.begin_frame(raw_input);

        self.draw_ui();

        // EGUIフレームの終了
        let egui_output = self.egui_ctx.end_frame();
        let paint_jobs = self.egui_ctx.tessellate(egui_output.shapes);

        // EGUIの描画
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUI Render Encoder"),
        });

        // 実際の描画
        self.egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // 画面をクリア
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // EGUIを描画
        self.egui_renderer
            .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// UIの描画
    fn draw_ui(&mut self) {
        let ctx = &self.egui_ctx;

        // トップパネル
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("シェーダーテスト環境");

                ui.separator();

                if ui.button("新規").clicked() {
                    self.create_new_test();
                }

                if ui.button("読込").clicked() {
                    // TODO: ファイル選択ダイアログ
                }

                if ui.button("保存").clicked() {
                    // TODO: ファイル保存ダイアログ
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let play_text = if self.is_playing { "■" } else { "▶" };
                    if ui.button(play_text).clicked() {
                        self.is_playing = !self.is_playing;
                    }

                    if ui.button("リセット").clicked() {
                        self.elapsed_time = 0.0;
                    }

                    ui.label(format!("時間: {:.2}秒", self.elapsed_time));
                });
            });
        });

        // 左パネル（テストリスト）
        egui::SidePanel::left("test_list_panel")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("テストケース");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, test) in self.test_cases.iter().enumerate() {
                        let is_selected =
                            Some(i) == self.current_test.as_ref().map(|_| self.current_test_index);
                        let response = ui.selectable_label(is_selected, &test.data.name);

                        if response.clicked() && !is_selected {
                            self.current_test = Some(test.clone());
                            self.current_test_index = i;
                            self.load_current_test();
                        }
                    }
                });

                ui.separator();
                if ui.button("追加").clicked() {
                    self.create_new_test();
                }
            });

        // 右パネル（パラメータ）
        egui::SidePanel::right("parameter_panel")
            .resizable(true)
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.heading("パラメータ");
                ui.separator();

                if let Some(test) = &mut self.current_test {
                    ui.horizontal(|ui| {
                        ui.label("名前:");
                        let mut name = test.data.name.clone();
                        if ui.text_edit_singleline(&mut name).changed() {
                            test.data.name = name;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("説明:");
                        let mut desc = test.data.description.clone();
                        if ui.text_edit_singleline(&mut desc).changed() {
                            test.data.description = desc;
                        }
                    });

                    ui.separator();
                    ui.heading("シェーダーのパラメータ");

                    if test.data.parameters.is_empty() {
                        ui.label("パラメータがありません");
                    } else {
                        for param in &test.data.parameters {
                            ui.horizontal(|ui| {
                                ui.label(&param.name);
                                ui.label(": ");
                                ui.label(&param.description);
                            });

                            let mut value = *self
                                .parameter_values
                                .get(&param.name)
                                .unwrap_or(&param.default);
                            if ui
                                .add(
                                    egui::Slider::new(&mut value, param.min..=param.max)
                                        .step_by(param.step as f64),
                                )
                                .changed()
                            {
                                self.parameter_values.insert(param.name.clone(), value);

                                // TODO: パラメータ値の更新をテストケースに反映
                            }
                        }
                    }
                } else {
                    ui.label("テストケースが選択されていません");
                }
            });

        // 下部パネル（シェーダーエディタ）
        egui::TopBottomPanel::bottom("shader_editor_panel")
            .resizable(true)
            .min_height(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("シェーダーエディタ");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("適用").clicked() {
                            self.apply_shader_code();
                        }
                    });
                });

                ui.separator();

                if self.current_test.is_some() {
                    let font = egui::TextStyle::Monospace.resolve(ui.style());
                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = egui::text::LayoutJob::default();

                        // 簡易的なシンタックスハイライト
                        for line in string.lines() {
                            // キーワードを色付け
                            let line_with_color = if line.trim().starts_with("//") {
                                // コメント
                                layout_job.append(
                                    line,
                                    0.0,
                                    egui::TextFormat::simple(
                                        font.clone(),
                                        Color32::from_rgb(100, 150, 100),
                                    ),
                                );
                            } else {
                                // キーワードを色付け
                                let keywords = [
                                    "fn",
                                    "struct",
                                    "var",
                                    "const",
                                    "let",
                                    "return",
                                    "if",
                                    "else",
                                    "for",
                                    "while",
                                    "switch",
                                    "case",
                                    "break",
                                    "continue",
                                    "@vertex",
                                    "@fragment",
                                    "@compute",
                                    "vec2",
                                    "vec3",
                                    "vec4",
                                    "mat2x2",
                                    "mat3x3",
                                    "mat4x4",
                                ];

                                let mut colored_line = line.to_string();
                                for keyword in keywords {
                                    if line.contains(keyword) {
                                        colored_line = colored_line
                                            .replace(keyword, &format!("##{keyword}##"));
                                    }
                                }

                                layout_job.append(
                                    line,
                                    0.0,
                                    egui::TextFormat::simple(font.clone(), Color32::WHITE),
                                );
                            };

                            layout_job.append(
                                "\n",
                                0.0,
                                egui::TextFormat::simple(font.clone(), Color32::WHITE),
                            );
                        }

                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    let mut editor = TextEdit::multiline(&mut self.shader_code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter);

                    if ui.add(editor).changed() {
                        self.shader_modified = true;
                    }

                    // エラーメッセージがあれば表示
                    if let Some(ref error) = self.compilation_error {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("エラー: ");
                            ui.colored_label(Color32::RED, error);
                        });
                    }
                } else {
                    ui.label("テストケースが選択されていません");
                }
            });

        // 中央パネル（レンダリング結果）
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("レンダリング結果");
            ui.separator();

            if let Some(texture_id) = self.output_texture_id {
                let test = self.current_test.as_ref().unwrap();
                let width = test.data.output_size.0 as f32;
                let height = test.data.output_size.1 as f32;

                // 中央に配置
                let available_size = ui.available_size();
                let scale = (available_size.x / width)
                    .min(available_size.y / height)
                    .min(1.0);
                let scaled_size = egui::vec2(width * scale, height * scale);

                let x = (available_size.x - scaled_size.x) * 0.5;
                let y = (available_size.y - scaled_size.y) * 0.5;

                ui.allocate_ui_at_rect(
                    egui::Rect::from_min_size(ui.min_rect().min + egui::vec2(x, y), scaled_size),
                    |ui| {
                        ui.image(texture_id, scaled_size);
                    },
                );

                // 境界線を描画
                ui.painter().rect_stroke(
                    egui::Rect::from_min_size(ui.min_rect().min + egui::vec2(x, y), scaled_size),
                    0.0,
                    Stroke::new(1.0, Color32::GRAY),
                );

                // 情報表示
                ui.horizontal(|ui| {
                    ui.label(format!("サイズ: {}x{}", width, height));
                    ui.label(format!("時間: {:.2}秒", self.elapsed_time));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No rendering output available");
                });
            }
        });
    }

    /// イベント処理
    pub fn handle_event(&mut self, window: &WinitWindow, event: &WindowEvent) -> bool {
        // 新しいeui-winitの方法でイベント処理
        let mut egui_state = egui_winit::State::new(
            egui_winit::EventLoopWindowTarget::from_window(window),
        );
        let response = egui_state.on_event(&self.egui_ctx, event);
        response.consumed
    }

    /// ウィンドウリサイズ処理
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // 必要に応じてリサイズ処理
    }
}
