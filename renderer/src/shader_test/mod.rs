//! シェーダーテスト環境
//!
//! シェーダーの開発とテストを支援するためのツールとフレームワークを提供します。
//!
//! 主な機能：
//! - インタラクティブなシェーダー編集と視覚的テスト
//! - 自動テスト検証
//! - CI/CD環境での実行
//! - ヘッドレスモードでのテスト実行

mod case;
mod headless;
mod runner;
// UIモジュールを一時的に無効化
// mod ui;
mod validator;

// 主要なコンポーネントをre-export
pub use case::TestCase;
pub use headless::HeadlessRunner;
pub use runner::ShaderTestRunner;
// UIモジュールを一時的に無効化
// pub use ui::ShaderTestUI;
pub use validator::{OutputValidator, ValidationResult};

/// パラメータ定義
///
/// シェーダーテストのパラメータを定義する構造体です。
#[derive(Debug, Clone)]
pub struct Parameter {
    /// パラメータ名
    pub name: String,
    /// パラメータの説明
    pub description: String,
    /// 最小値
    pub min: f32,
    /// 最大値
    pub max: f32,
    /// デフォルト値
    pub default: f32,
    /// ステップ値
    pub step: f32,
}

impl Parameter {
    /// 新しいパラメータを作成
    pub fn new(name: &str, description: &str, min: f32, max: f32, default: f32, step: f32) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            min,
            max,
            default,
            step,
        }
    }
}

/// テスト設定
///
/// シェーダーテストの設定を定義する構造体です。
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// テスト名
    pub name: String,
    /// テストの説明
    pub description: String,
    /// 使用するシェーダー（ファイルパスまたはWGSLコード）
    pub shader: ShaderSource,
    /// テストパラメータ
    pub parameters: Vec<Parameter>,
    /// テスト出力サイズ
    pub output_size: (u32, u32),
    /// バックグラウンドカラー
    pub background_color: [f32; 4],
}

/// シェーダーソース
///
/// シェーダーのソースを表す列挙型です。
#[derive(Debug, Clone)]
pub enum ShaderSource {
    /// 組み込みシェーダー名
    BuiltIn(String),
    /// WGSLコード
    Code(String),
    /// ファイルパス
    File(std::path::PathBuf),
}

/// シェーダーテストの結果
///
/// テスト実行結果を表す構造体です。
#[derive(Debug)]
pub struct TestResult {
    /// テスト名
    pub test_name: String,
    /// テスト成功フラグ
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error_message: Option<String>,
    /// 出力イメージ（成功時）
    pub output_image: Option<image::RgbaImage>,
    /// テスト実行時間（ミリ秒）
    pub execution_time_ms: u64,
}

/// シェーダーテスト環境の設定
///
/// テスト環境全体の設定を表す構造体です。
#[derive(Debug, Clone)]
pub struct TestEnvironmentConfig {
    /// ウィンドウタイトル
    pub window_title: String,
    /// ウィンドウサイズ
    pub window_size: (u32, u32),
    /// テスト出力領域サイズ
    pub render_size: (u32, u32),
    /// テストディレクトリ
    pub test_directory: Option<std::path::PathBuf>,
    /// 自動実行フラグ
    pub auto_run: bool,
    /// ヘッドレスモードフラグ
    pub headless: bool,
    /// ログレベル
    pub log_level: log::LevelFilter,
}

impl Default for TestEnvironmentConfig {
    fn default() -> Self {
        Self {
            window_title: "シェーダーテスト環境".to_string(),
            window_size: (1280, 720),
            render_size: (512, 512),
            test_directory: None,
            auto_run: false,
            headless: false,
            log_level: log::LevelFilter::Info,
        }
    }
}
