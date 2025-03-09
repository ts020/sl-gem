// UIシェーダー
// UI要素のレンダリングに使用するシェーダー

// 頂点シェーダーの入力
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

// インスタンスデータ
struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) tex_coords_min: vec2<f32>,
    @location(7) tex_coords_max: vec2<f32>,
    @location(8) color: vec4<f32>,
    @location(9) ui_type: u32,  // UI要素のタイプ（0=テクスチャ、1=単色、2=グラデーション）
};

// 頂点シェーダーの出力
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ui_type: u32,
    @location(3) position_in_quad: vec2<f32>,  // 四角形内の相対位置（0,0～1,1）
};

// ユニフォームバッファ
struct Uniforms {
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,  // スクリーンサイズ
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// テクスチャとサンプラー
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// 頂点シェーダー
@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // モデル行列を再構築
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // ワールド座標を計算
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    
    // 出力を構築
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * world_position;
    
    // テクスチャ座標を計算
    let tex_coords = mix(
        instance.tex_coords_min,
        instance.tex_coords_max,
        vertex.tex_coords
    );
    out.tex_coords = tex_coords;
    
    // 色を設定
    out.color = instance.color;
    
    // UI要素のタイプを設定
    out.ui_type = instance.ui_type;
    
    // 四角形内の相対位置を設定
    out.position_in_quad = vertex.tex_coords;
    
    return out;
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 常にテクスチャをサンプリング（制御フローの一様性のため）
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // 各UIタイプごとの色を計算
    // テクスチャ (0u)
    let texture_color = tex_color * in.color;
    
    // 単色 (1u)
    let solid_color = in.color;
    
    // グラデーション (2u)
    let gradient_factor = in.position_in_quad.y;
    let dark_color = in.color * 0.7;
    let gradient_color = mix(in.color, dark_color, gradient_factor);
    
    // 枠線付き四角形 (3u)
    let border_width = 0.05;
    let is_border = (in.position_in_quad.x < border_width ||
                     in.position_in_quad.x > 1.0 - border_width ||
                     in.position_in_quad.y < border_width ||
                     in.position_in_quad.y > 1.0 - border_width);
    let border_color = vec4<f32>(in.color.rgb * 0.5, in.color.a);
    let bordered_rect_color = select(in.color, border_color, is_border);
    
    // UIタイプに応じて最終的な色を選択
    var final_color: vec4<f32>;
    if (in.ui_type == 0u) {
        final_color = texture_color;
    } else if (in.ui_type == 1u) {
        final_color = solid_color;
    } else if (in.ui_type == 2u) {
        final_color = gradient_color;
    } else if (in.ui_type == 3u) {
        final_color = bordered_rect_color;
    } else {
        final_color = in.color; // デフォルト
    }
    
    // アルファ値が低すぎる場合は破棄
    if (final_color.a < 0.01) {
        discard;
    }
    
    return final_color;
}