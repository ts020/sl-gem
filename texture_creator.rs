use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // タイルセットの作成（シンプルな色付きブロックのタイル）
    let tile_size = 32;
    let tiles_count = 8;
    let width = tile_size * tiles_count;
    let height = tile_size;

    // PNGヘッダー（8x1 RGB8形式のシンプルなPNG画像）
    let mut png_data = Vec::new();
    // 画像データを追加する代わりに、1x1の色付きピクセルを作成してファイルに保存する

    // 単色のシンプルな画像を作成
    let mut file = File::create("game/assets/textures/tiles/default_tileset.png")?;
    let tile_colors = [
        [0, 255, 0],     // 緑色(平地)
        [0, 153, 0],    // 深緑色(森林)
        [128, 77, 0],   // 茶色(山地)
        [0, 0, 204],    // 青色(水域)
        [179, 179, 0],  // 黄色(道路)
        [179, 179, 179],// 灰色(都市)
        [204, 0, 204],  // 紫色(拠点)
        [255, 255, 255] // 白色(予備)
    ];

    // RGBでシンプルなPPMフォーマットを使用（テキストベースの画像フォーマット）
    let mut ppm_data = format!("P6
{} {}
255
", width, height);
    let mut pixel_data = Vec::new();

    // 各タイルの色を設定
    for y in 0..height {
        for x in 0..width {
            // どのタイルに属するかを計算
            let tile_index = x / tile_size;
            if tile_index < tile_colors.len() {
                // タイルの色を取得
                let color = tile_colors[tile_index];
                pixel_data.push(color[0]);
                pixel_data.push(color[1]);
                pixel_data.push(color[2]);
            } else {
                // 範囲外は黒
                pixel_data.push(0);
                pixel_data.push(0);
                pixel_data.push(0);
            }
        }
    }

    // ヘッダーとピクセルデータを書き込む
    file.write_all(ppm_data.as_bytes())?;
    file.write_all(&pixel_data)?;

    // ユニットセットの作成
    let mut file = File::create("game/assets/textures/units/default_unitset.png")?;
    let unit_colors = [
        [255, 0, 0],     // 赤チーム
        [0, 0, 255],     // 青チーム
        [0, 204, 0],     // 緑チーム
        [255, 255, 0],   // 黄チーム
        [128, 0, 128],   // 紫チーム
        [0, 255, 255],   // シアンチーム
        [255, 165, 0],   // オレンジチーム
        [255, 255, 255]  // 白色(予備)
    ];

    // 同様にPPMフォーマットのユニットセットを作成
    let mut ppm_data = format!("P6
{} {}
255
", width, height);
    let mut pixel_data = Vec::new();

    // 各ユニットを描画
    for y in 0..height {
        for x in 0..width {
            // どのユニット枠に属するかを計算
            let unit_index = x / tile_size;
            let unit_x = x 