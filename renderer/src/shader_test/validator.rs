//! 出力検証モジュール
//!
//! シェーダーテストの出力を検証するための機能を提供します。

use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

/// 検証結果
///
/// シェーダーテスト出力の検証結果を表す構造体です。
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 検証成功フラグ
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error_message: Option<String>,
    /// 差異点の座標リスト（ピクセル座標）
    pub diff_points: Vec<(u32, u32)>,
}

impl ValidationResult {
    /// 成功結果を作成
    pub fn success() -> Self {
        Self {
            success: true,
            error_message: None,
            diff_points: Vec::new(),
        }
    }

    /// 失敗結果を作成
    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            error_message: Some(message.to_string()),
            diff_points: Vec::new(),
        }
    }

    /// 差異点を含む失敗結果を作成
    pub fn with_diff_points(message: &str, diff_points: Vec<(u32, u32)>) -> Self {
        Self {
            success: false,
            error_message: Some(message.to_string()),
            diff_points,
        }
    }
}

/// 出力検証器
///
/// シェーダーテスト出力を検証するトレイトです。
pub trait OutputValidator {
    /// 出力を検証する
    fn validate(&self, output: &[u8], width: u32, height: u32) -> ValidationResult;
}

/// ピクセル値検証器
///
/// 特定のピクセル位置での値を検証する検証器です。
pub struct PixelValidator {
    /// 検証するピクセルの座標リスト
    pub points: Vec<(u32, u32)>,
    /// 期待される色値
    pub expected_colors: Vec<[u8; 4]>,
    /// 許容誤差
    pub tolerance: u8,
}

impl PixelValidator {
    /// 新しいピクセル検証器を作成
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            expected_colors: Vec::new(),
            tolerance: 5,
        }
    }

    /// 検証ポイントを追加
    pub fn add_point(&mut self, x: u32, y: u32, color: [u8; 4]) {
        self.points.push((x, y));
        self.expected_colors.push(color);
    }

    /// 許容誤差を設定
    pub fn set_tolerance(&mut self, tolerance: u8) {
        self.tolerance = tolerance;
    }
}

impl OutputValidator for PixelValidator {
    fn validate(&self, output: &[u8], width: u32, height: u32) -> ValidationResult {
        let mut diff_points = Vec::new();

        // 出力データからRgbaImageを作成
        let output_image = match RgbaImage::from_raw(width, height, output.to_vec()) {
            Some(img) => img,
            None => return ValidationResult::failure("出力データから画像を作成できません"),
        };

        // 各検証ポイントをチェック
        for (i, &(x, y)) in self.points.iter().enumerate() {
            if x >= width || y >= height {
                return ValidationResult::failure(&format!(
                    "検証ポイント ({}, {}) が画像範囲外です ({}x{})",
                    x, y, width, height
                ));
            }

            let expected = self.expected_colors[i];
            let actual = output_image.get_pixel(x, y).0;

            // 色の差異を計算
            let diff_r = (expected[0] as i32 - actual[0] as i32).abs() as u8;
            let diff_g = (expected[1] as i32 - actual[1] as i32).abs() as u8;
            let diff_b = (expected[2] as i32 - actual[2] as i32).abs() as u8;
            let diff_a = (expected[3] as i32 - actual[3] as i32).abs() as u8;

            // 許容誤差より大きい差異があれば失敗
            if diff_r > self.tolerance
                || diff_g > self.tolerance
                || diff_b > self.tolerance
                || diff_a > self.tolerance
            {
                diff_points.push((x, y));
            }
        }

        if diff_points.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::with_diff_points(
                &format!("{} ピクセルで色の不一致があります", diff_points.len()),
                diff_points,
            )
        }
    }
}

/// 画像比較検証器
///
/// 基準画像との比較を行う検証器です。
pub struct ImageCompareValidator {
    /// 基準画像
    pub reference_image: RgbaImage,
    /// 許容誤差（0.0～1.0）
    pub tolerance: f32,
    /// 最大差異ピクセル数
    pub max_diff_points: usize,
}

impl ImageCompareValidator {
    /// 新しい画像比較検証器を作成
    pub fn new(reference_path: &Path, tolerance: f32, max_diff_points: usize) -> Result<Self> {
        // 基準画像を読み込む
        let reference_image = image::open(reference_path)?.to_rgba8();

        Ok(Self {
            reference_image,
            tolerance: tolerance.clamp(0.0, 1.0),
            max_diff_points,
        })
    }

    /// 基準画像を設定
    pub fn with_reference_image(
        reference_image: RgbaImage,
        tolerance: f32,
        max_diff_points: usize,
    ) -> Self {
        Self {
            reference_image,
            tolerance: tolerance.clamp(0.0, 1.0),
            max_diff_points,
        }
    }
}

impl OutputValidator for ImageCompareValidator {
    fn validate(&self, output: &[u8], width: u32, height: u32) -> ValidationResult {
        // 出力データからRgbaImageを作成
        let output_image = match RgbaImage::from_raw(width, height, output.to_vec()) {
            Some(img) => img,
            None => return ValidationResult::failure("出力データから画像を作成できません"),
        };

        // 画像サイズが一致しない場合はエラー
        if self.reference_image.width() != width || self.reference_image.height() != height {
            return ValidationResult::failure(&format!(
                "基準画像のサイズ ({}x{}) と出力画像のサイズ ({}x{}) が一致しません",
                self.reference_image.width(),
                self.reference_image.height(),
                width,
                height
            ));
        }

        let mut diff_points = Vec::new();
        let tolerance_value = (self.tolerance * 255.0) as u8;

        // 各ピクセルを比較
        for y in 0..height {
            for x in 0..width {
                let reference_pixel = self.reference_image.get_pixel(x, y).0;
                let output_pixel = output_image.get_pixel(x, y).0;

                // 色の差異を計算
                let diff_r = (reference_pixel[0] as i32 - output_pixel[0] as i32).abs() as u8;
                let diff_g = (reference_pixel[1] as i32 - output_pixel[1] as i32).abs() as u8;
                let diff_b = (reference_pixel[2] as i32 - output_pixel[2] as i32).abs() as u8;
                let diff_a = (reference_pixel[3] as i32 - output_pixel[3] as i32).abs() as u8;

                // 許容誤差より大きい差異があれば記録
                if diff_r > tolerance_value
                    || diff_g > tolerance_value
                    || diff_b > tolerance_value
                    || diff_a > tolerance_value
                {
                    diff_points.push((x, y));

                    // 最大差異ピクセル数を超えたら早期リターン
                    if diff_points.len() > self.max_diff_points {
                        return ValidationResult::with_diff_points(
                            &format!("{}ピクセル以上で色の不一致があります", self.max_diff_points),
                            diff_points,
                        );
                    }
                }
            }
        }

        if diff_points.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::with_diff_points(
                &format!("{}ピクセルで色の不一致があります", diff_points.len()),
                diff_points,
            )
        }
    }
}

/// 差分画像を生成
///
/// 出力画像と基準画像の差分を可視化した画像を生成します。
pub fn generate_diff_image(
    output: &[u8],
    width: u32,
    height: u32,
    reference_image: &RgbaImage,
    diff_points: &[(u32, u32)],
) -> Result<RgbaImage> {
    // 出力データからRgbaImageを作成
    let output_image = match RgbaImage::from_raw(width, height, output.to_vec()) {
        Some(img) => img,
        None => return Err(anyhow::anyhow!("出力データから画像を作成できません")),
    };

    // 差分画像を作成
    let mut diff_image = RgbaImage::new(width, height);

    // 基本的には出力画像をコピー
    for y in 0..height {
        for x in 0..width {
            diff_image.put_pixel(x, y, *output_image.get_pixel(x, y));
        }
    }

    // 差異のあるピクセルは赤色でマーク
    let highlight_color = Rgba([255, 0, 0, 255]);
    for &(x, y) in diff_points {
        if x < width && y < height {
            diff_image.put_pixel(x, y, highlight_color);
        }
    }

    Ok(diff_image)
}

/// 統計検証
///
/// シェーダー出力の統計的特性を検証します。
pub struct StatisticalValidator {
    /// 期待される平均輝度（0.0～1.0）
    pub expected_avg_luminance: Option<f32>,
    /// 期待される最小輝度（0.0～1.0）
    pub expected_min_luminance: Option<f32>,
    /// 期待される最大輝度（0.0～1.0）
    pub expected_max_luminance: Option<f32>,
    /// 許容誤差
    pub tolerance: f32,
}

impl StatisticalValidator {
    /// 新しい統計検証器を作成
    pub fn new() -> Self {
        Self {
            expected_avg_luminance: None,
            expected_min_luminance: None,
            expected_max_luminance: None,
            tolerance: 0.05,
        }
    }

    /// 平均輝度を設定
    pub fn set_avg_luminance(&mut self, value: f32) {
        self.expected_avg_luminance = Some(value.clamp(0.0, 1.0));
    }

    /// 最小輝度を設定
    pub fn set_min_luminance(&mut self, value: f32) {
        self.expected_min_luminance = Some(value.clamp(0.0, 1.0));
    }

    /// 最大輝度を設定
    pub fn set_max_luminance(&mut self, value: f32) {
        self.expected_max_luminance = Some(value.clamp(0.0, 1.0));
    }
}

impl OutputValidator for StatisticalValidator {
    fn validate(&self, output: &[u8], width: u32, height: u32) -> ValidationResult {
        // 出力データからRgbaImageを作成
        let output_image = match RgbaImage::from_raw(width, height, output.to_vec()) {
            Some(img) => img,
            None => return ValidationResult::failure("出力データから画像を作成できません"),
        };

        let mut sum_luminance: f32 = 0.0;
        let mut min_luminance: f32 = 1.0;
        let mut max_luminance: f32 = 0.0;

        // 各ピクセルの輝度を計算
        for y in 0..height {
            for x in 0..width {
                let pixel = output_image.get_pixel(x, y).0;

                // RGBを輝度に変換 (ITU-R BT.709の係数)
                let luminance = (0.2126 * pixel[0] as f32
                    + 0.7152 * pixel[1] as f32
                    + 0.0722 * pixel[2] as f32)
                    / 255.0;

                sum_luminance += luminance;
                min_luminance = min_luminance.min(luminance);
                max_luminance = max_luminance.max(luminance);
            }
        }

        let avg_luminance = sum_luminance / (width * height) as f32;
        let mut error_messages = Vec::new();

        // 平均輝度の検証
        if let Some(expected) = self.expected_avg_luminance {
            let diff = (expected - avg_luminance).abs();
            if diff > self.tolerance {
                error_messages.push(format!(
                    "平均輝度: 期待値={:.3}, 実際値={:.3}, 差={:.3}",
                    expected, avg_luminance, diff
                ));
            }
        }

        // 最小輝度の検証
        if let Some(expected) = self.expected_min_luminance {
            let diff = (expected - min_luminance).abs();
            if diff > self.tolerance {
                error_messages.push(format!(
                    "最小輝度: 期待値={:.3}, 実際値={:.3}, 差={:.3}",
                    expected, min_luminance, diff
                ));
            }
        }

        // 最大輝度の検証
        if let Some(expected) = self.expected_max_luminance {
            let diff = (expected - max_luminance).abs();
            if diff > self.tolerance {
                error_messages.push(format!(
                    "最大輝度: 期待値={:.3}, 実際値={:.3}, 差={:.3}",
                    expected, max_luminance, diff
                ));
            }
        }

        if error_messages.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(&error_messages.join(", "))
        }
    }
}
