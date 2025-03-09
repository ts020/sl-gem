//! カメラシステム
//! 
//! マップのビューを管理し、スクロールやズーム機能を提供します。

use glam::{Mat4, Vec2, Vec3};

/// カメラ
/// 
/// 2Dマップのビューを管理するカメラシステムです。
/// 位置、ズーム、回転などのビュー変換を処理します。
#[derive(Debug, Clone)]
pub struct Camera {
    /// カメラの位置（ワールド座標）
    pub position: Vec2,
    /// ズーム倍率（1.0が標準）
    pub zoom: f32,
    /// 回転角度（ラジアン）
    pub rotation: f32,
    /// ビューポートの幅
    pub viewport_width: f32,
    /// ビューポートの高さ
    pub viewport_height: f32,
}

impl Camera {
    /// 新しいカメラを作成
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            viewport_width,
            viewport_height,
        }
    }

    /// ビューポートのサイズを更新
    pub fn update_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// ビュー行列を計算
    /// 
    /// カメラの位置、回転、ズームに基づいてビュー行列を計算します。
    pub fn view_matrix(&self) -> Mat4 {
        // 移動行列（カメラの位置の逆方向に移動）
        let translation = Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));
        
        // 回転行列（カメラの回転の逆方向に回転）
        let rotation = Mat4::from_rotation_z(-self.rotation);
        
        // ズーム行列（カメラのズームに応じてスケーリング）
        let scale = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));
        
        // 行列を合成（順序に注意：スケール→回転→移動）
        scale * rotation * translation
    }

    /// 射影行列を計算
    /// 
    /// 2D正投影行列を計算します。
    pub fn projection_matrix(&self) -> Mat4 {
        // 正投影行列（2D）
        let aspect_ratio = self.viewport_width / self.viewport_height;
        let left = -aspect_ratio;
        let right = aspect_ratio;
        let bottom = -1.0;
        let top = 1.0;
        let near = -1.0;
        let far = 1.0;
        
        Mat4::orthographic_rh(left, right, bottom, top, near, far)
    }

    /// ビュー射影行列を計算
    /// 
    /// ビュー行列と射影行列を合成した行列を計算します。
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// スクロール
    /// 
    /// カメラを指定された量だけスクロールします。
    pub fn scroll(&mut self, delta_x: f32, delta_y: f32) {
        // ズーム倍率に応じてスクロール量を調整
        let scroll_speed = 1.0 / self.zoom;
        self.position.x += delta_x * scroll_speed;
        self.position.y += delta_y * scroll_speed;
    }

    /// ズーム
    /// 
    /// カメラのズーム倍率を変更します。
    pub fn zoom(&mut self, factor: f32) {
        self.zoom *= factor;
        
        // ズーム値の制限（極端な値にならないように）
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }

    /// スクリーン座標からワールド座標への変換
    /// 
    /// スクリーン上の座標（ピクセル）をワールド座標に変換します。
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // スクリーン座標を正規化座標に変換
        let normalized_x = (screen_pos.x / self.viewport_width) * 2.0 - 1.0;
        let normalized_y = 1.0 - (screen_pos.y / self.viewport_height) * 2.0; // Y軸は反転
        
        // 正規化座標をワールド座標に変換
        let normalized_pos = Vec2::new(normalized_x, normalized_y);
        
        // ビュー射影行列の逆行列を計算
        let inverse_view_proj = self.view_projection_matrix().inverse();
        
        // 正規化座標にビュー射影行列の逆行列を適用
        let world_pos_homogeneous = inverse_view_proj * Vec3::new(normalized_pos.x, normalized_pos.y, 0.0).extend(1.0);
        
        // 同次座標から2D座標に変換
        Vec2::new(world_pos_homogeneous.x, world_pos_homogeneous.y)
    }

    /// ワールド座標からスクリーン座標への変換
    /// 
    /// ワールド座標をスクリーン上の座標（ピクセル）に変換します。
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // ワールド座標にビュー射影行列を適用
        let clip_pos = self.view_projection_matrix() * Vec3::new(world_pos.x, world_pos.y, 0.0).extend(1.0);
        
        // 同次座標から正規化座標に変換
        let normalized_x = clip_pos.x / clip_pos.w;
        let normalized_y = clip_pos.y / clip_pos.w;
        
        // 正規化座標をスクリーン座標に変換
        let screen_x = (normalized_x + 1.0) * 0.5 * self.viewport_width;
        let screen_y = (1.0 - normalized_y) * 0.5 * self.viewport_height; // Y軸は反転
        
        Vec2::new(screen_x, screen_y)
    }

    /// MapGUIのスクロール値からカメラ位置を設定
    /// 
    /// MapGUIのスクロール値（ピクセル単位）からカメラの位置を設定します。
    pub fn set_from_map_gui_scroll(&mut self, scroll_x: i32, scroll_y: i32, tile_size: u32) {
        // MapGUIのスクロール値はピクセル単位なので、タイルサイズで割ってタイル単位に変換
        let tile_size_f = tile_size as f32;
        let tile_x = scroll_x as f32 / tile_size_f;
        let tile_y = scroll_y as f32 / tile_size_f;
        
        // カメラ位置を設定（Y軸は反転する可能性があるため注意）
        self.position = Vec2::new(tile_x, tile_y);
    }

    /// MapGUIのズーム値からカメラのズームを設定
    /// 
    /// MapGUIのズーム値からカメラのズーム倍率を設定します。
    pub fn set_from_map_gui_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}