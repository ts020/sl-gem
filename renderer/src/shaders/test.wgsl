// テスト用シェーダー
// シェーダーテスト環境で使用する汎用的なシェーダー

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
    @location(6) color: vec4<f32>,
};

// 頂点シェーダーの出力
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) position: vec2<f32>,   // シェーダー内での位置情報（エフェクト用）
};

// ユニフォームバッファ
struct Uniforms {
    // 基本的な変換行列
    view_proj: mat4x4<f32>,
    
    // テスト用のパラメータ
    time: f32,                // 時間（アニメーション用）
    param1: f32,              // ユーザー定義パラメータ1
    param2: f32,              // ユーザー定義パラメータ2
    param3: f32,              // ユーザー定義パラメータ3
    
    // エフェクト制御フラグ
    mode: u32,                // レンダリングモード（0=基本、1=波形、2=色相循環、3=モザイク、4=歪み）
    enable_texture: u32,      // テクスチャ有効フラグ
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
) -> VertexOutput {
    // デフォルトのモデル行列（アイデンティティ行列）
    let model_matrix = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );

    // 基本的な位置計算
    var modified_position = vertex.position;
    
    // モード1：波形アニメーション
    if (uniforms.mode == 1u) {
        let wave = sin(vertex.position.x * uniforms.param1 + uniforms.time * uniforms.param2) * uniforms.param3;
        modified_position.y += wave;
    }
    
    // ワールド座標を計算
    let world_position = model_matrix * vec4<f32>(modified_position, 1.0);
    
    // 出力を構築
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * world_position;
    out.tex_coords = vertex.tex_coords;
    out.color = vec4<f32>(1.0, 0.5, 0.0, 1.0); // デフォルトのオレンジ色
    out.position = vertex.position.xy; // 元の位置を保存
    
    return out;
}

// HSV to RGB変換関数
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let h_prime = h * 6.0;
    let x = c * (1.0 - abs(fract(h_prime / 2.0) * 2.0 - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    
    if (h_prime < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h_prime < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h_prime < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h_prime < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h_prime < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + vec3<f32>(m, m, m);
}

// フラグメントシェーダー
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    
    // テクスチャが有効な場合はサンプリング
    if (uniforms.enable_texture == 1u) {
        tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    }
    
    // 基本色（インスタンスの色とテクスチャの色の組み合わせ）
    var final_color = in.color * tex_color;
    
    // モード別の特殊効果
    if (uniforms.mode == 0u) {
        // 基本モード：そのまま
    } else if (uniforms.mode == 1u) {
        // 波形モード：すでに頂点シェーダーで適用済み
    } else if (uniforms.mode == 2u) {
        // 色相循環モード
        let hue = fract(uniforms.time * 0.1 + dot(in.position, vec2<f32>(0.5, 0.5)));
        let rgb = hsv_to_rgb(hue, uniforms.param1, uniforms.param2);
        final_color = vec4<f32>(rgb * tex_color.rgb * in.color.rgb, in.color.a * tex_color.a);
    } else if (uniforms.mode == 3u) {
        // モザイクモード
        let cell_size = max(uniforms.param1, 0.001);
        let cell_x = floor(in.tex_coords.x / cell_size) * cell_size + (cell_size * 0.5);
        let cell_y = floor(in.tex_coords.y / cell_size) * cell_size + (cell_size * 0.5);
        let cell_tex_coords = vec2<f32>(cell_x, cell_y);
        
        if (uniforms.enable_texture == 1u) {
            let mosaic_color = textureSample(t_diffuse, s_diffuse, cell_tex_coords);
            final_color = in.color * mosaic_color;
        }
    } else if (uniforms.mode == 4u) {
        // 歪みモード
        let distortion_strength = uniforms.param1;
        let distortion_speed = uniforms.param2;
        let distortion_scale = max(uniforms.param3, 0.001);
        
        let time_offset = uniforms.time * distortion_speed;
        let distortion_x = sin(in.tex_coords.y * distortion_scale + time_offset) * distortion_strength;
        let distortion_y = cos(in.tex_coords.x * distortion_scale + time_offset) * distortion_strength;
        
        let distorted_coords = vec2<f32>(
            in.tex_coords.x + distortion_x,
            in.tex_coords.y + distortion_y
        );
        
        if (uniforms.enable_texture == 1u) {
            let distorted_color = textureSample(t_diffuse, s_diffuse, distorted_coords);
            final_color = in.color * distorted_color;
        }
    }
    
    // アルファ値が低すぎる場合は破棄
    if (final_color.a < 0.01) {
        discard;
    }
    
    return final_color;
}