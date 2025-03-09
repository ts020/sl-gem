//! テストケース定義モジュール
//!
//! シェーダーテストのテストケースを定義するための構造体と関数を提供します。

use super::{Parameter, ShaderSource, ValidationResult};
use anyhow::Result;
use glam::{Mat4, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// RONファイルからのデシリアライズに使用する設定構造体
#[derive(Debug, Deserialize)]
pub struct TestCaseConfig {
    /// テスト名
    pub name: String,
    
    /// テストの説明
    pub description: String,
    
    /// シェーダーソース
    pub shader: ShaderSourceConfig,
    
    /// 頂点データ（位置x,y,z, テクスチャ座標u,v）
    pub vertex_data: Vec<(f32, f32, f32, f32, f32)>,
    
    /// インデックスデータ（オプション）
    pub index_data: Option<Vec<u16>>,
    
    /// インスタンスデータ（オプション）
    #[serde(default)]
    pub instance_data: Option<Vec<InstanceDataConfig>>,
    
    /// テクスチャパス（オプション）
    #[serde(default)]
    pub texture_path: Option<String>,
    
    /// テストパラメータ
    #[serde(default)]
    pub parameters: Vec<ParameterConfig>,
    
    /// 出力サイズ
    pub output_size: (u32, u32),
    
    /// バックグラウンドカラー (R,G,B,A)
    pub background_color: (f32, f32, f32, f32),
    
    /// 許容差異（0.0-1.0）
    #[serde(default = "default_tolerance")]
    pub tolerance: f32,
}

/// デフォルトの許容差異値
fn default_tolerance() -> f32 {
    0.01
}

/// RONファイルからのデシリアライズに使用するパラメータ設定構造体
#[derive(Debug, Deserialize)]
pub struct ParameterConfig {
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
    
    /// 増減ステップ
    pub step: f32,
}

/// インスタンスデータの設定
#[derive(Debug, Deserialize)]
pub struct InstanceDataConfig {
    /// モデル行列（16個の要素を一次元配列で表現）
    #[serde(default = "default_model_matrix")]
    pub model_matrix: [f32; 16],
    
    /// カラー (RGBA)
    #[serde(default = "default_color")]
    pub color: [f32; 4],
}

fn default_model_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,  // 1行目
        0.0, 1.0, 0.0, 0.0,  // 2行目
        0.0, 0.0, 1.0, 0.0,  // 3行目
        0.0, 0.0, 0.0, 1.0,  // 4行目
    ]
}

fn default_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]  // 白色
}

/// シェーダーソース設定
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ShaderSourceConfig {
    /// 組み込みシェーダー名
    BuiltIn(String),
    /// ファイルパス
    File(String),
    /// シェーダーコード
    Code(String),
}

impl ToString for ShaderSourceConfig {
    fn to_string(&self) -> String {
        match self {
            ShaderSourceConfig::BuiltIn(name) => name.clone(),
            ShaderSourceConfig::File(path) => path.clone(),
            ShaderSourceConfig::Code(code) => code.clone(),
        }
    }
}

/// テストケース
///
/// シェーダーのテストケースを定義する構造体です。
// 個別フィールドにderiveを適用するために構造体を分割
#[derive(Debug, Clone)]
pub struct TestCaseData {
    /// テスト名
    pub name: String,

    /// テストの説明
    pub description: String,

    /// シェーダーソース
    pub shader: ShaderSource,

    /// 頂点データ
    pub vertex_data: Vec<super::super::Vertex>,

    /// インデックスデータ
    pub index_data: Option<Vec<u16>>,

    /// テクスチャパス
    pub texture_path: Option<PathBuf>,

    /// ユニフォームデータ
    pub uniforms: HashMap<String, UniformValue>,

    /// テストパラメータ
    pub parameters: Vec<Parameter>,

    /// 出力サイズ
    pub output_size: (u32, u32),

    /// バックグラウンドカラー
    pub background_color: [f32; 4],

    /// 基準画像パス（比較検証用）
    pub reference_image: Option<PathBuf>,

    /// 許容差異（ピクセル単位の差異許容範囲、0.0-1.0）
    pub tolerance: f32,
}

pub struct TestCase {
    // 自動導出可能なデータ
    pub data: TestCaseData,

    /// 検証関数 - CloneもDebugも実装できないため別で管理
    pub validation_function: Option<Box<dyn Fn(&[u8], u32, u32) -> ValidationResult + Send + Sync>>,
}

/// ユニフォーム値
///
/// シェーダーに渡すユニフォーム値を表す列挙型です。
#[derive(Debug, Clone)]
pub enum UniformValue {
    /// 単一の32ビット浮動小数点数
    Float(f32),
    /// 2次元ベクトル
    Vec2(Vec2),
    /// 3次元ベクトル
    Vec3(Vec3),
    /// 4次元ベクトル
    Vec4(Vec4),
    /// 4x4行列
    Mat4(Mat4),
    /// 32ビット整数
    Int(i32),
    /// 32ビット符号なし整数
    Uint(u32),
    /// 論理値
    Bool(bool),
}

impl Clone for TestCase {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            // Fn型はCloneできないので検証関数はクローン時に捨てる
            validation_function: None,
        }
    }
}

impl std::fmt::Debug for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("data", &self.data)
            .field("validation_function", &"<function>")
            .finish()
    }
}

impl TestCase {
    /// 新しいテストケースを作成
    pub fn new(name: &str) -> Self {
        Self {
            data: TestCaseData {
                name: name.to_string(),
                description: String::new(),
                shader: ShaderSource::BuiltIn("test".to_string()),
                vertex_data: create_quad_vertices(),
                index_data: Some(create_quad_indices()),
                texture_path: None,
                uniforms: HashMap::new(),
                parameters: Vec::new(),
                output_size: (512, 512),
                background_color: [0.0, 0.0, 0.0, 1.0],
                reference_image: None,
                tolerance: 0.01,
            },
            validation_function: None,
        }
    }

    /// 説明を設定
    pub fn with_description(mut self, description: &str) -> Self {
        self.data.description = description.to_string();
        self
    }

    /// シェーダーを設定
    pub fn with_shader(mut self, shader: &str) -> Self {
        // 拡張子がwgslであればファイルパスとして扱う
        if shader.ends_with(".wgsl") {
            self.data.shader = ShaderSource::File(PathBuf::from(shader));
        }
        // 特殊なキーワードであれば組み込みシェーダーとして扱う
        else if ["test", "tile", "unit", "ui"].contains(&shader) {
            self.data.shader = ShaderSource::BuiltIn(shader.to_string());
        }
        // それ以外はシェーダーコードとして扱う
        else {
            self.data.shader = ShaderSource::Code(shader.to_string());
        }
        self
    }

    /// 頂点データを設定
    pub fn with_vertex_data(mut self, vertices: Vec<super::super::Vertex>) -> Self {
        self.data.vertex_data = vertices;
        self
    }

    /// インデックスデータを設定
    pub fn with_index_data(mut self, indices: Vec<u16>) -> Self {
        self.data.index_data = Some(indices);
        self
    }

    /// テクスチャを設定
    pub fn with_texture(mut self, path: &str) -> Self {
        self.data.texture_path = Some(PathBuf::from(path));
        self
    }

    /// ユニフォームを追加
    pub fn with_uniform<T: Into<UniformValue>>(mut self, name: &str, value: T) -> Self {
        self.data.uniforms.insert(name.to_string(), value.into());
        self
    }

    /// パラメータを追加
    pub fn with_parameter(mut self, parameter: Parameter) -> Self {
        self.data.parameters.push(parameter);
        self
    }

    /// 出力サイズを設定
    pub fn with_output_size(mut self, width: u32, height: u32) -> Self {
        self.data.output_size = (width, height);
        self
    }

    /// バックグラウンドカラーを設定
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.data.background_color = [r, g, b, a];
        self
    }

    /// 検証関数を設定
    pub fn with_validation<F>(mut self, f: F) -> Self
    where
        F: Fn(&[u8], u32, u32) -> ValidationResult + Send + Sync + 'static,
    {
        self.validation_function = Some(Box::new(f));
        self
    }

    /// 基準画像を設定
    pub fn with_reference_image(mut self, path: &str, tolerance: f32) -> Self {
        self.data.reference_image = Some(PathBuf::from(path));
        self.data.tolerance = tolerance;
        self
    }

    /// テスト実行を行うためのバイナリデータを生成（ユニフォームバッファ用）
    pub fn create_uniform_buffer(&self, time: f32) -> Vec<u8> {
        // 基本的なユニフォームデータ（view_proj行列と時間）
        let mut data = Vec::new();

        // view_proj行列（デフォルトはアイデンティティ行列）
        let view_proj = self
            .data
            .uniforms
            .get("view_proj")
            .map(|v| match v {
                UniformValue::Mat4(m) => *m,
                _ => Mat4::IDENTITY,
            })
            .unwrap_or(Mat4::IDENTITY);

        // 行列データを追加
        data.extend_from_slice(bytemuck::cast_slice(&view_proj.to_cols_array()));

        // 時間を追加
        data.extend_from_slice(bytemuck::cast_slice(&[time]));

        // パラメータ値を追加（最大3つまで）
        let param_values: Vec<f32> = self
            .data
            .parameters
            .iter()
            .map(|p| p.default)
            .take(3)
            .collect();

        let param_count = param_values.len();
        data.extend_from_slice(bytemuck::cast_slice(&param_values));

        // 不足しているパラメータを0.0で埋める
        for _ in 0..(3 - param_count) {
            data.extend_from_slice(bytemuck::cast_slice(&[0.0f32]));
        }

        // モードを追加（デフォルトは0）
        let mode = self
            .data
            .uniforms
            .get("mode")
            .map(|v| match v {
                UniformValue::Uint(u) => *u,
                _ => 0u32,
            })
            .unwrap_or(0u32);

        data.extend_from_slice(bytemuck::cast_slice(&[mode]));

        // テクスチャ有効フラグを追加
        let enable_texture = if self.data.texture_path.is_some() {
            1u32
        } else {
            0u32
        };
        data.extend_from_slice(bytemuck::cast_slice(&[enable_texture]));

        // 16バイト境界にアラインメントするためのパディングを追加
        // 現在のサイズは88バイト、96バイトになるまでパディングを追加
        let padding_bytes = 8; // 96 - 88 = 8バイト
        data.extend_from_slice(&[0u8; 8]);

        data
    }

    /// テストケースをRONファイルから読み込む
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        use std::fs::File;
        use std::io::Read;
        use ron::de::from_reader;
        
        // ファイルを開いて内容を読み込む
        let mut file = File::open(path.as_ref())?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        
        // RON形式データをパース
        let config = from_reader::<_, TestCaseConfig>(data.as_bytes())?;
        
        // TestCaseConfigからTestCaseを構築
        let mut test_case = TestCase::new(&config.name)
            .with_description(&config.description)
            .with_shader(&config.shader.to_string())
            .with_output_size(config.output_size.0, config.output_size.1)
            .with_background_color(
                config.background_color.0,
                config.background_color.1,
                config.background_color.2,
                config.background_color.3,
            );
        
        // 頂点データを設定
        let vertices = config.vertex_data.iter().map(|v| super::super::Vertex {
            position: [v.0, v.1, v.2],
            tex_coords: [v.3, v.4],
        }).collect();
        test_case = test_case.with_vertex_data(vertices);
        
        // インデックスデータがあれば設定
        if let Some(indices) = config.index_data {
            test_case = test_case.with_index_data(indices);
        }
        
        // パラメータを設定
        for param in config.parameters {
            test_case = test_case.with_parameter(Parameter::new(
                &param.name,
                &param.description,
                param.min,
                param.max,
                param.default,
                param.step,
            ));
        }
        
        // テクスチャパスが指定されていれば設定
        if let Some(texture_path) = config.texture_path {
            test_case = test_case.with_texture(&texture_path);
        }
        
        // 許容誤差を設定
        test_case.data.tolerance = config.tolerance;
        
        // インスタンスデータを処理する（存在する場合）
        if let Some(instance_configs) = config.instance_data {
            // インスタンスごとにTileInstanceに変換
            for instance_config in instance_configs {
                // モデル行列を2D配列に変換
                let model_matrix = [
                    [instance_config.model_matrix[0], instance_config.model_matrix[1], instance_config.model_matrix[2], instance_config.model_matrix[3]],
                    [instance_config.model_matrix[4], instance_config.model_matrix[5], instance_config.model_matrix[6], instance_config.model_matrix[7]],
                    [instance_config.model_matrix[8], instance_config.model_matrix[9], instance_config.model_matrix[10], instance_config.model_matrix[11]],
                    [instance_config.model_matrix[12], instance_config.model_matrix[13], instance_config.model_matrix[14], instance_config.model_matrix[15]],
                ];
                
                // ユニフォームバッファにインスタンスデータを追加
                test_case = test_case.with_uniform("has_instances", 1u32);
            }
        }
        
        Ok(test_case)
    }
    
    /// インスタンスデータを取得
    pub fn instance_data(&self) -> Option<&[super::super::TileInstance]> {
        // 実装されていない場合はNoneを返す
        None
    }

    // アクセサメソッド - 便利のため追加
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn description(&self) -> &str {
        &self.data.description
    }

    pub fn shader(&self) -> &ShaderSource {
        &self.data.shader
    }

    pub fn output_size(&self) -> (u32, u32) {
        self.data.output_size
    }

    pub fn background_color(&self) -> [f32; 4] {
        self.data.background_color
    }

    pub fn vertex_data(&self) -> &[super::super::Vertex] {
        &self.data.vertex_data
    }

    pub fn index_data(&self) -> Option<&[u16]> {
        self.data.index_data.as_ref().map(|v| v.as_slice())
    }

    pub fn texture_path(&self) -> Option<&PathBuf> {
        self.data.texture_path.as_ref()
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.data.parameters
    }

    pub fn tolerance(&self) -> f32 {
        self.data.tolerance
    }
}

/// 標準的な四角形の頂点データを作成
pub fn create_quad_vertices() -> Vec<super::super::Vertex> {
    vec![
        super::super::Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [0.0, 1.0],
        }, // 左下
        super::super::Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [1.0, 1.0],
        }, // 右下
        super::super::Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [1.0, 0.0],
        }, // 右上
        super::super::Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [0.0, 0.0],
        }, // 左上
    ]
}

/// 標準的な四角形のインデックスデータを作成
pub fn create_quad_indices() -> Vec<u16> {
    vec![
        0, 1, 2, // 三角形1
        0, 2, 3, // 三角形2
    ]
}

/// 単位インスタンスデータを作成
pub fn create_unit_instance(color: [f32; 4]) -> super::super::TileInstance {
    super::super::TileInstance {
        model_matrix: Mat4::IDENTITY.to_cols_array_2d(),
        tex_coords_min: [0.0, 0.0],
        tex_coords_max: [1.0, 1.0],
        color,
    }
}

// UniformValue型の変換実装
impl From<f32> for UniformValue {
    fn from(val: f32) -> Self {
        UniformValue::Float(val)
    }
}

impl From<Vec2> for UniformValue {
    fn from(val: Vec2) -> Self {
        UniformValue::Vec2(val)
    }
}

impl From<Vec3> for UniformValue {
    fn from(val: Vec3) -> Self {
        UniformValue::Vec3(val)
    }
}

impl From<Vec4> for UniformValue {
    fn from(val: Vec4) -> Self {
        UniformValue::Vec4(val)
    }
}

impl From<Mat4> for UniformValue {
    fn from(val: Mat4) -> Self {
        UniformValue::Mat4(val)
    }
}

impl From<i32> for UniformValue {
    fn from(val: i32) -> Self {
        UniformValue::Int(val)
    }
}

impl From<u32> for UniformValue {
    fn from(val: u32) -> Self {
        UniformValue::Uint(val)
    }
}

impl From<bool> for UniformValue {
    fn from(val: bool) -> Self {
        UniformValue::Bool(val)
    }
}

/// ビルトインテストケースを作成
pub fn create_builtin_testcases() -> Vec<TestCase> {
    vec![
        // 基本的なカラーテスト
        TestCase::new("basic_color")
            .with_description("基本的な単色シェーダーテスト")
            .with_shader("test")
            .with_background_color(0.0, 0.0, 0.0, 1.0)
            .with_uniform("mode", 0u32)
            .with_parameter(Parameter::new("color_r", "赤成分", 0.0, 1.0, 1.0, 0.01))
            .with_parameter(Parameter::new("color_g", "緑成分", 0.0, 1.0, 0.5, 0.01))
            .with_parameter(Parameter::new("color_b", "青成分", 0.0, 1.0, 0.0, 0.01)),
        // 波形アニメーションテスト
        TestCase::new("wave_animation")
            .with_description("波形アニメーションエフェクトのテスト")
            .with_shader("test")
            .with_background_color(0.1, 0.1, 0.2, 1.0)
            .with_uniform("mode", 1u32)
            .with_parameter(Parameter::new("frequency", "周波数", 1.0, 20.0, 5.0, 0.1))
            .with_parameter(Parameter::new("speed", "速度", 0.1, 5.0, 1.0, 0.1))
            .with_parameter(Parameter::new("amplitude", "振幅", 0.01, 0.2, 0.05, 0.01)),
        // テクスチャテスト
        TestCase::new("texture_test")
            .with_description("テクスチャマッピングのテスト")
            .with_shader("test")
            .with_texture("tests/textures/test_pattern.ppm") // 新しく生成したテストパターン
            .with_background_color(0.0, 0.0, 0.0, 1.0)
            .with_uniform("mode", 0u32)
            .with_uniform("enable_texture", 1u32),
    ]
}
