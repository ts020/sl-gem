use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> std::io::Result<()> {
    // チェッカーボードパターンのテスト画像を生成（256x256ピクセル）
    let width = 256;
    let height = 256;
    let path = Path::new("tests/textures/test_pattern.ppm");
    
    let mut file = File::create(path)?;
    
    // PPMヘッダー
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", width, height)?;
    writeln!(file, "255")?;
    
    // チェッカーボードパターンのピクセルデータを生成
    let cell_size = 32; // 各マスのサイズ
    
    for y in 0..height {
        for x in 0..width {
            let cell_x = x / cell_size;
            let cell_y = y / cell_size;
            
            // 交互に色を変える
            let color = if (cell_x + cell_y) % 2 == 0 {
                [255, 0, 0] // 赤
            } else {
                [0, 0, 255] // 青
            };
            
            // RGBデータを書き込む
            file.write_all(&color)?;
        }
    }
    
    println!("テストパターン画像を生成しました: {}", path.display());
    Ok(())
}