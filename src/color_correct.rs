use image::Pixel;

pub fn color_correct(skin_data: &mut image::RgbaImage) {
    for row in skin_data.rows_mut() {
        for pixel in row {
            let channels = pixel.channels_mut();
            // 0..3 - don't apply to alpha channel
            for i in 0..3 {
                let pixel = &mut channels[i];
                
                // gamma brightening by a factor of 1.385, bounded to [0, 184]
                *pixel = ((((*pixel as f64) / 255.0).powf(1.0f64 / 1.385f64) * 255.0) * 184.0 / 255.0) as u8;
            }
        }
    }
}
