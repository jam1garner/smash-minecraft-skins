use ordered_float::NotNan;
use image::imageops::{overlay, resize, Nearest};
use image::{GenericImageView, Pixel, ImageFormat};
use color_thief::{get_palette, ColorFormat, Color};

fn color_distance(x: Color, y: Color) -> f32 {
    (
        (y.r as f32 - x.r as f32).powi(2) +
        (y.g as f32 - x.g as f32).powi(2) +
        (y.b as f32 - x.b as f32).powi(2)
    ).sqrt()
}

pub fn gen_stock_image(img: &image::RgbaImage) -> image::RgbaImage {
    let (width, height) = img.dimensions();
    assert_eq!(width, height);

    let pixel_scale = width / 64;
    
    let face = img.view(8 * pixel_scale, 8 * pixel_scale, 8 * pixel_scale, 8 * pixel_scale);
    let mut face = resize(&face, 40, 40, Nearest);

    let mut outline = image::load_from_memory_with_format(
        include_bytes!("stock_outline.png"),
        ImageFormat::Png
    ).unwrap().into_rgba();

    let buf = face.as_raw();
    let pallete = get_palette(buf, ColorFormat::Rgba, 10, 4).unwrap();
    
    for pixel in face.pixels_mut() {
        let channels = pixel.channels_mut();

        let color = match channels {
            &mut [r, g, b, ..] => Color::new(r, g, b),
            _ => panic!("Invalid number of channels")
        };

        let closest = pallete
            .iter()
            .min_by_key(|x| NotNan::new(color_distance(**x, color)).unwrap())
            .unwrap();

        channels[0] = closest.r;
        channels[1] = closest.g;
        channels[2] = closest.b;
    }

    overlay(&mut outline, &face, 12, 12);

    outline
}
