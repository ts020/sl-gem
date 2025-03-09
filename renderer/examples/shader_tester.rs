//! シェーダーテスト環境
//!
//! インタラクティブなシェーダーテスト環境を起動するエントリポイントです。

use anyhow::Result;
use clap::{ArgAction, Parser};
use renderer::shader_test::{HeadlessRunner, TestEnvironmentConfig};
use std::path::PathBuf;

/// シェーダーテスト環境のコマンドライン引数
#[derive(Parser, Debug)]
#[clap(author, version, about = "シェーダーテスト環境")]
struct CliArgs {
    /// テストケースディレクトリ
    #[clap(short, long, value_name = "DIR")]
    test_dir: Option<PathBuf>,

    /// ヘッドレスモード（視覚的出力なし）
    #[clap(short = 'H', long, action = ArgAction::SetTrue)]
    headless: bool,

    /// HTMLレポート出力パス
    #[clap(short, long, value_name = "FILE")]
    report: Option<PathBuf>,

    /// ウィンドウ幅
    #[clap(long, default_value = "1280")]
    width: u32,

    /// ウィンドウ高さ
    #[clap(long, default_value = "720")]
    height: u32,

    /// テスト自動実行
    #[clap(short, long, action = ArgAction::SetTrue)]
    auto_run: bool,

    /// 詳細ログ出力
    #[clap(short, long, action = ArgAction::SetTrue)]
    verbose: bool,
}

/// メイン関数
#[tokio::main]
async fn main() -> Result<()> {
    // コマンドライン引数をパース
    let args = CliArgs::parse();

    // ログ設定
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(if args.verbose { "debug" } else { "info" }),
    )
    .init();

    // テスト環境の設定
    let config = TestEnvironmentConfig {
        window_title: "シェーダーテスト環境".to_string(),
        window_size: (args.width, args.height),
        render_size: (512, 512),
        test_directory: args.test_dir,
        auto_run: args.auto_run,
        headless: true, // 常にヘッドレスモードで実行
        log_level: if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
    };

    // ヘッドレスモードで実行
    run_headless(config, args.report).await
}

/// ヘッドレスモードでの実行
async fn run_headless(config: TestEnvironmentConfig, report_path: Option<PathBuf>) -> Result<()> {
    // HeadlessRunnerは既にインポート済み

    // テストディレクトリの設定
    let test_dir = config
        .test_directory
        .unwrap_or_else(|| PathBuf::from("tests/shader_tests"));

    // 出力ディレクトリの設定
    let output_dir = PathBuf::from("target/shader_test_output");

    // ヘッドレスランナーを作成
    let mut runner = HeadlessRunner::new(test_dir, output_dir);
    runner.set_verbose(config.log_level == log::LevelFilter::Debug);

    // テストケースを読み込む
    runner.load_tests()?;

    // テストを実行
    let success = runner.run_tests().await?;

    // HTMLレポートを生成
    if let Some(path) = report_path {
        runner.generate_html_report(path)?;
    }

    // 成功したかどうかでプロセス終了コードを設定
    if !success {
        std::process::exit(1);
    }

    Ok(())
}

/* インタラクティブモードは一時的に無効化されています

/// インタラクティブモードでの実行
async fn run_interactive(config: TestEnvironmentConfig) -> Result<()> {
    unimplemented!("インタラクティブモードは現在無効化されています");
}
*/
