//! ヘッドレステスト実行環境
//!
//! CI/CD環境などで視覚的出力なしでシェーダーテストを実行するための機能を提供します。

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{ShaderTestRunner, TestCase, TestResult, ValidationResult};

/// ヘッドレスランナー
///
/// CI/CD環境などでシェーダーテストを自動実行するためのモジュールです。
pub struct HeadlessRunner {
    /// テストケースのディレクトリパス
    test_dir: PathBuf,
    /// 出力ディレクトリパス
    output_dir: PathBuf,
    /// テストケースのリスト
    test_cases: Vec<TestCase>,
    /// 基準イメージの格納ディレクトリ
    reference_dir: PathBuf,
    /// テスト結果
    results: Vec<TestResult>,
    /// タイムアウト時間（秒）
    timeout: f32,
    /// 詳細ログ出力フラグ
    verbose: bool,
}

impl HeadlessRunner {
    /// 新しいヘッドレスランナーを作成
    pub fn new<P: AsRef<Path>>(test_dir: P, output_dir: P) -> Self {
        let test_dir = test_dir.as_ref().to_path_buf();
        let output_dir = output_dir.as_ref().to_path_buf();
        let reference_dir = test_dir.join("references");

        Self {
            test_dir,
            output_dir,
            reference_dir,
            test_cases: Vec::new(),
            results: Vec::new(),
            timeout: 30.0, // デフォルトは30秒
            verbose: false,
        }
    }

    /// 詳細ログ出力を設定
    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    /// タイムアウト時間を設定
    pub fn set_timeout(&mut self, timeout_seconds: f32) -> &mut Self {
        self.timeout = timeout_seconds;
        self
    }

    /// テストケースを読み込む
    pub fn load_tests(&mut self) -> Result<()> {
        // テストディレクトリが存在しない場合は作成
        if !self.test_dir.exists() {
            fs::create_dir_all(&self.test_dir)?;
        }

        // 出力ディレクトリが存在しない場合は作成
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)?;
        }

        // 基準イメージディレクトリが存在しない場合は作成
        if !self.reference_dir.exists() {
            fs::create_dir_all(&self.reference_dir)?;
        }

        // テストファイルを検索（.jsonファイル）
        let test_files = fs::read_dir(&self.test_dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if let Some(ext) = entry.path().extension() {
                    ext == "json"
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        // テストファイルが見つからない場合はデフォルトテストを追加
        if test_files.is_empty() {
            info!("テストファイルが見つかりません。デフォルトテストを使用します。");
            self.test_cases = super::case::create_builtin_testcases();
            return Ok(());
        }

        // テストファイルを読み込む
        for entry in test_files {
            let path = entry.path();
            match self.load_test_from_file(&path) {
                Ok(test) => {
                    if self.verbose {
                        info!("テストケースを読み込みました: {}", test.name());
                    }
                    self.test_cases.push(test);
                }
                Err(err) => {
                    warn!("テストケースの読み込みに失敗しました {:?}: {}", path, err);
                }
            }
        }

        Ok(())
    }

    /// ファイルからテストケースを読み込む
    fn load_test_from_file<P: AsRef<Path>>(&self, path: P) -> Result<TestCase> {
        // TODO: JSONからのテストケース読み込み実装
        // 現在は未実装のためエラーを返す
        Err(anyhow::anyhow!("テストケースのJSONロードは未実装です"))
    }

    /// テストを実行
    pub async fn run_tests(&mut self) -> Result<bool> {
        if self.test_cases.is_empty() {
            info!("実行するテストケースがありません");
            return Ok(true);
        }

        info!("{}個のテストケースを実行します", self.test_cases.len());

        // 結果をクリア
        self.results.clear();

        // WGPUコンテキストを初期化
        let wgpu_context = super::super::WgpuContext::new_headless(512, 512).await?;

        // テストランナーを作成
        let mut runner = ShaderTestRunner::new_with_context(wgpu_context);

        // 各テストケースを実行
        let mut success_count = 0;
        let total_count = self.test_cases.len();

        for (i, test_case) in self.test_cases.iter().enumerate() {
            info!(
                "[{}/{}] テスト実行中: {}",
                i + 1,
                total_count,
                test_case.data.name.clone()
            );

            let start_time = Instant::now();
            let result = self.run_single_test(&mut runner, test_case, i);
            let execution_time = start_time.elapsed();

            match result {
                Ok(test_result) => {
                    if test_result.success {
                        success_count += 1;
                        info!(
                            "✅ テスト成功: {} ({:.2}ms)",
                            test_case.name(),
                            execution_time.as_millis()
                        );
                    } else {
                        if let Some(ref err) = test_result.error_message {
                            error!("❌ テスト失敗: {}: {}", test_case.name(), err);
                        } else {
                            error!("❌ テスト失敗: {}", test_case.name());
                        }
                    }
                    self.results.push(test_result);
                }
                Err(err) => {
                    error!("⚠️ テスト実行エラー: {}: {}", test_case.name(), err);
                    self.results.push(TestResult {
                        test_name: test_case.name().to_string(),
                        success: false,
                        error_message: Some(format!("実行エラー: {}", err)),
                        output_image: None,
                        execution_time_ms: execution_time.as_millis() as u64,
                    });
                }
            }
        }

        // テスト結果のサマリーを表示
        info!(
            "テスト結果: {}/{} 成功 ({}%)",
            success_count,
            total_count,
            (success_count as f32 / total_count as f32 * 100.0) as u32
        );

        Ok(success_count == total_count)
    }

    /// 単一テストケースを実行
    fn run_single_test(
        &self,
        runner: &mut ShaderTestRunner,
        test_case: &TestCase,
        index: usize,
    ) -> Result<TestResult> {
        // テストケースを設定
        runner.set_test_case(test_case.clone());

        // テスト実行開始時間
        let start_time = Instant::now();

        // リソースを初期化
        runner.initialize_resources()?;

        // タイムアウト処理付きでテストを実行
        let result = runner.run();

        // 実行時間
        let execution_time = start_time.elapsed();

        // タイムアウトチェック
        if execution_time.as_secs_f32() > self.timeout {
            warn!("テストがタイムアウトしました: {}", test_case.name());
            return Ok(TestResult {
                test_name: test_case.name().to_string(),
                success: false,
                error_message: Some(format!("タイムアウト（{}秒以上）", self.timeout)),
                output_image: None,
                execution_time_ms: execution_time.as_millis() as u64,
            });
        }

        // 出力画像を取得
        let output_image = match runner.get_output_image() {
            Ok(img) => Some(img),
            Err(err) => {
                warn!("出力画像の取得に失敗: {}", err);
                None
            }
        };

        // 出力を保存
        if let Some(ref img) = output_image {
            let output_path = self
                .output_dir
                .join(format!("test_{:03}_output.png", index));
            if let Err(err) = img.save(&output_path) {
                warn!("出力画像の保存に失敗: {}: {}", output_path.display(), err);
            }
        }

        // 検証関数があれば実行
        if let Some(validation_fn) = &test_case.validation_function {
            if let Some(ref img) = output_image {
                let output_data = img.as_raw();
                let width = img.width();
                let height = img.height();

                let validation_result = validation_fn(&output_data, width, height);

                if !validation_result.success {
                    // 差分のある部分を可視化した画像を生成
                    if let Some(ref error_msg) = validation_result.error_message {
                        warn!("検証エラー: {}", error_msg);
                    }

                    return Ok(TestResult {
                        test_name: test_case.name().to_string(),
                        success: false,
                        error_message: validation_result.error_message,
                        output_image: output_image.clone(),
                        execution_time_ms: execution_time.as_millis() as u64,
                    });
                }
            }
        }

        // 基準画像との比較
        if let Some(ref_path) = &test_case.data.reference_image {
            let reference_path = self.reference_dir.join(ref_path);

            if reference_path.exists() {
                if let Some(ref output_img) = output_image {
                    let reference_img = match image::open(&reference_path) {
                        Ok(img) => img.to_rgba8(),
                        Err(err) => {
                            warn!(
                                "基準画像の読み込みに失敗: {}: {}",
                                reference_path.display(),
                                err
                            );
                            return Ok(TestResult {
                                test_name: test_case.name().to_string(),
                                success: false,
                                error_message: Some(format!("基準画像の読み込みに失敗: {}", err)),
                                output_image: Some(output_img.clone()),
                                execution_time_ms: execution_time.as_millis() as u64,
                            });
                        }
                    };

                    // 画像サイズが一致しない場合はエラー
                    if output_img.width() != reference_img.width()
                        || output_img.height() != reference_img.height()
                    {
                        return Ok(TestResult {
                            test_name: test_case.name().to_string(),
                            success: false,
                            error_message: Some(format!(
                                "画像サイズが一致しません: 出力={}x{}, 基準={}x{}",
                                output_img.width(),
                                output_img.height(),
                                reference_img.width(),
                                reference_img.height()
                            )),
                            output_image: Some(output_img.clone()),
                            execution_time_ms: execution_time.as_millis() as u64,
                        });
                    }

                    // ピクセル比較
                    let mut diff_count = 0;
                    let width = output_img.width();
                    let height = output_img.height();
                    let tolerance = (test_case.tolerance() * 255.0) as u8;

                    for y in 0..height {
                        for x in 0..width {
                            let output_pixel = output_img.get_pixel(x, y).0;
                            let reference_pixel = reference_img.get_pixel(x, y).0;

                            let diff_r =
                                (output_pixel[0] as i32 - reference_pixel[0] as i32).abs() as u8;
                            let diff_g =
                                (output_pixel[1] as i32 - reference_pixel[1] as i32).abs() as u8;
                            let diff_b =
                                (output_pixel[2] as i32 - reference_pixel[2] as i32).abs() as u8;
                            let diff_a =
                                (output_pixel[3] as i32 - reference_pixel[3] as i32).abs() as u8;

                            if diff_r > tolerance
                                || diff_g > tolerance
                                || diff_b > tolerance
                                || diff_a > tolerance
                            {
                                diff_count += 1;
                            }
                        }
                    }

                    // 差異が多すぎる場合はエラー
                    let max_diff_pixels = (width * height) as f32 * 0.01; // 1%まで許容
                    if diff_count as f32 > max_diff_pixels {
                        return Ok(TestResult {
                            test_name: test_case.name().to_string(),
                            success: false,
                            error_message: Some(format!(
                                "画像に差異があります: {}ピクセル ({}%)",
                                diff_count,
                                (diff_count as f32 / (width * height) as f32 * 100.0) as u32
                            )),
                            output_image: Some(output_img.clone()),
                            execution_time_ms: execution_time.as_millis() as u64,
                        });
                    }
                }
            } else {
                warn!("基準画像が見つかりません: {}", reference_path.display());
                // 初回実行時は参照画像として保存
                if let Some(ref output_img) = output_image {
                    // 親ディレクトリが存在しない場合は作成
                    if let Some(parent) = reference_path.parent() {
                        if !parent.exists() {
                            if let Err(err) = fs::create_dir_all(parent) {
                                warn!("ディレクトリの作成に失敗: {}: {}", parent.display(), err);
                            }
                        }
                    }

                    // 画像を保存
                    if let Err(err) = output_img.save(&reference_path) {
                        warn!(
                            "基準画像の保存に失敗: {}: {}",
                            reference_path.display(),
                            err
                        );
                    } else {
                        info!("基準画像を作成しました: {}", reference_path.display());
                    }
                }
            }
        }

        // 成功結果を返す
        Ok(TestResult {
            test_name: test_case.name().to_string(),
            success: true,
            error_message: None,
            output_image,
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }

    /// テスト結果を取得
    pub fn get_results(&self) -> &[TestResult] {
        &self.results
    }

    /// HTMLレポートを生成
    pub fn generate_html_report<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let output_path = output_path.as_ref();

        // HTMLの開始部分
        let mut html = r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>シェーダーテスト結果レポート</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
        h1 { color: #333; }
        .summary { margin-bottom: 20px; padding: 10px; background-color: #f5f5f5; border-radius: 5px; }
        .test-list { list-style-type: none; padding: 0; }
        .test-item { margin-bottom: 20px; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .test-item.success { border-left: 5px solid #4CAF50; }
        .test-item.failure { border-left: 5px solid #F44336; }
        .test-header { display: flex; justify-content: space-between; align-items: center; }
        .test-name { font-weight: bold; font-size: 18px; }
        .test-result { padding: 5px 10px; border-radius: 3px; color: white; }
        .success-tag { background-color: #4CAF50; }
        .failure-tag { background-color: #F44336; }
        .test-details { margin-top: 10px; }
        .test-image { margin-top: 15px; max-width: 100%; }
        .error-message { color: #F44336; margin-top: 10px; font-family: monospace; padding: 10px; background-color: #ffebee; border-radius: 3px; }
        .execution-time { color: #666; font-size: 14px; }
    </style>
</head>
<body>
    <h1>シェーダーテスト結果レポート</h1>
"#.to_string();

        // サマリー部分
        let success_count = self.results.iter().filter(|r| r.success).count();
        let total_count = self.results.len();
        let success_rate = if total_count > 0 {
            (success_count as f32 / total_count as f32 * 100.0) as u32
        } else {
            0
        };

        html.push_str(&format!(
            r#"
    <div class="summary">
        <h2>テスト結果サマリー</h2>
        <p>実行テスト数: <strong>{}</strong></p>
        <p>成功: <strong>{}</strong> ({}%)</p>
        <p>失敗: <strong>{}</strong></p>
        <p>実行日時: <strong>{}</strong></p>
    </div>
"#,
            total_count,
            success_count,
            success_rate,
            total_count - success_count,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));

        // テストリスト
        html.push_str(
            r#"
    <h2>テスト詳細</h2>
    <ul class="test-list">
"#,
        );

        for result in &self.results {
            let status_class = if result.success { "success" } else { "failure" };
            let status_tag = if result.success {
                "success-tag"
            } else {
                "failure-tag"
            };
            let status_text = if result.success { "成功" } else { "失敗" };

            html.push_str(&format!(
                r#"
        <li class="test-item {}">
            <div class="test-header">
                <div class="test-name">{}</div>
                <div class="test-result {}">{}</div>
            </div>
            <div class="test-details">
                <div class="execution-time">実行時間: {}ms</div>
"#,
                status_class, result.test_name, status_tag, status_text, result.execution_time_ms
            ));

            // エラーメッセージがあれば表示
            if let Some(ref error_message) = result.error_message {
                html.push_str(&format!(
                    r#"
                <div class="error-message">{}</div>
"#,
                    error_message
                ));
            }

            // 出力画像があれば埋め込み
            if let Some(ref image) = result.output_image {
                // 画像をBase64エンコード
                let mut buffer = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut buffer);

                if let Err(err) = image.write_to(&mut cursor, image::ImageFormat::Png) {
                    warn!("画像のエンコードに失敗: {}", err);
                } else {
                    let base64_image = base64::encode(&buffer);
                    html.push_str(&format!(
                        r#"
                <div class="test-image">
                    <img src="data:image/png;base64,{}" alt="テスト出力画像" />
                </div>
"#,
                        base64_image
                    ));
                }
            }

            html.push_str(
                r#"
            </div>
        </li>
"#,
            );
        }

        // HTMLの終了部分
        html.push_str(
            r#"
    </ul>
</body>
</html>
"#,
        );

        // ファイルに書き込み
        fs::write(output_path, html)?;
        info!("HTMLレポートを生成しました: {}", output_path.display());

        Ok(())
    }
}
