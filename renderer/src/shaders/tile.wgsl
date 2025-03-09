// タイルシェーダー
// 2Dタイルマップのレンダリングに使用するシェーダー

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
};

// 頂点シェーダーの出力
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// ユニフォームバッファ
struct Uniforms {
    view_proj: mat4x4<f32>,
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
    // 入力のテクスチャ座標（0.0～1.0）をインスタンスのテクスチャ範囲に変換
    let tex_coords = mix(
        instance.tex_coords_min,
        instance.tex_coords_max,
        vertex.tex_coords
    );
    out.tex_coords = tex_coords;
    
    // 色を設定
    out.color = instance.color;
    
    return out;
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // テクスチャからカラーを取得
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // テクスチャの色とインスタンスの色を乗算
    return tex_color * in.color;
}