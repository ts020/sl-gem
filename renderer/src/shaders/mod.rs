//! シェーダーモジュール
//!
//! WGSLシェーダーの管理を担当します。

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::Arc;

// 組み込みシェーダー
/// タイルシェーダー
pub const TILE_SHADER: &str = include_str!("tile.wgsl");

/// ユニットシェーダー
pub const UNIT_SHADER: &str = include_str!("unit.wgsl");

/// UIシェーダー
pub const UI_SHADER: &str = include_str!("ui.wgsl");

/// テスト用シェーダー
pub const TEST_SHADER: &str = include_str!("test.wgsl");

/// シェーダーローダー
///
/// シェーダーファイルの読み込みと管理を行います。
pub struct ShaderLoader {
    device: Arc<wgpu::Device>,
}

impl ShaderLoader {
    /// 新しいシェーダーローダーを作成
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self { device }
    }

    /// 文字列からシェーダーモジュールを作成
    pub fn create_shader_module_from_str(
        &self,
        source: &str,
        label: Option<&str>,
    ) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
    }

    /// ファイルからシェーダーモジュールを作成
    pub fn create_shader_module_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        label: Option<&str>,
    ) -> Result<wgpu::ShaderModule> {
        let source = fs::read_to_string(path)?;
        Ok(self.create_shader_module_from_str(&source, label))
    }

    /// タイルシェーダーモジュールを作成
    pub fn create_tile_shader(&self) -> wgpu::ShaderModule {
        self.create_shader_module_from_str(TILE_SHADER, Some("Tile Shader"))
    }

    /// ユニットシェーダーモジュールを作成
    pub fn create_unit_shader(&self) -> wgpu::ShaderModule {
        self.create_shader_module_from_str(UNIT_SHADER, Some("Unit Shader"))
    }

    /// UIシェーダーモジュールを作成
    pub fn create_ui_shader(&self) -> wgpu::ShaderModule {
        self.create_shader_module_from_str(UI_SHADER, Some("UI Shader"))
    }

    /// テスト用シェーダーモジュールを作成
    pub fn create_test_shader(&self) -> wgpu::ShaderModule {
        self.create_shader_module_from_str(TEST_SHADER, Some("Test Shader"))
    }
}

/// シェーダーコンパイラ
///
/// シェーダーのコンパイルと検証を行います。
/// テスト環境で使用します。
pub struct ShaderCompiler {
    device: Arc<wgpu::Device>,
}

impl ShaderCompiler {
    /// 新しいシェーダーコンパイラを作成
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self { device }
    }

    /// シェーダーコードをコンパイル
    pub fn compile(&self, source: &str) -> Result<wgpu::ShaderModule, String> {
        use std::panic::AssertUnwindSafe;

        let device = self.device.clone(); // クローンして所有権を共有
        let result = std::panic::catch_unwind(AssertUnwindSafe(move || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Dynamic Shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        }));

        match result {
            Ok(module) => Ok(module),
            Err(e) => {
                if let Some(err_string) = e.downcast_ref::<String>() {
                    Err(err_string.clone())
                } else {
                    Err("Unknown shader compilation error".to_string())
                }
            }
        }
    }

    /// シェーダーのバリデーション
    pub fn validate(&self, source: &str) -> Result<(), String> {
        match self.compile(source) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::DeviceDescriptor;

    async fn setup_device() -> Arc<wgpu::Device> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, _) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await
            .unwrap();
        Arc::new(device)
    }

    #[tokio::test]
    async fn test_compile_tile_shader() {
        let device = setup_device().await;
        let loader = ShaderLoader::new(device.clone());

        // タイルシェーダーをコンパイル
        let shader_module = loader.create_tile_shader();
        assert!(shader_module.global_id() != wgpu::ShaderModuleId::DUMMY);
    }

    #[tokio::test]
    async fn test_compile_invalid_shader() {
        let device = setup_device().await;
        let compiler = ShaderCompiler::new(device);

        // 無効なシェーダーコード
        let invalid_source = r#"
            @vertex
            fn vs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }
            
            // 構文エラー
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                return color // セミコロンがない
            }
        "#;

        let result = compiler.validate(invalid_source);
        assert!(result.is_err());
    }
}
