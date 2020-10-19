use image::GenericImage;

/// Copy from one area to another, then flip the resulting area
fn copy_flipped(image: &mut image::RgbaImage, from_pos: (u32, u32), size: (u32, u32), to_pos: (u32, u32)) {
    let (x, y) = from_pos;
    let (width, height) = size;
    let (to_x, to_y) = to_pos;
    image.copy_within(image::math::Rect { x, y, width, height }, to_x, to_y);
    
    image::imageops::flip_horizontal_in_place(&mut image.sub_image(to_x, to_y, width, height));
}

/// Copy from one area to another, shift horizontally (with wrapping) N pixels, then flip the resulting area
fn copy_rotated_right_flipped(image: &mut image::RgbaImage, from_pos: (u32, u32), size: (u32, u32), to_pos: (u32, u32), shift: u32) {
    let (x, y) = from_pos;
    let (width, height) = size;
    let (to_x, to_y) = to_pos;
    let shift = shift % width;
    
    image.copy_within(image::math::Rect { x, y, width: width - shift, height }, to_x + shift, to_y);
    image.copy_within(image::math::Rect { x: (width - shift), y, width: shift, height }, to_x, to_y);
    
    image::imageops::flip_horizontal_in_place(&mut image.sub_image(to_x, to_y, width, height));
}

pub fn convert_to_modern_skin(skin_data: &image::RgbaImage) -> image::RgbaImage {
    let scale = skin_data.width() / 64;

    let mut new_skin = image::RgbaImage::new(64 * scale, 64 * scale);

    new_skin.copy_from(skin_data, 0, 0).unwrap();

    let arm_size: (u32, u32) = (4 * scale, 4 * scale);

    // copy and flip the top of leg
    copy_flipped(&mut new_skin, (4 * scale, 16 * scale), arm_size, (20 * scale, 48 * scale));

    // copy and flip the bottom of leg
    copy_flipped(&mut new_skin, (8 * scale, 16 * scale), arm_size, (24 * scale, 48 * scale));

    // copy and flip the top of arm
    copy_flipped(&mut new_skin, (44 * scale, 16 * scale), arm_size, (36 * scale, 48 * scale));

    // copy and flip the bottom of arm
    copy_flipped(&mut new_skin, (48 * scale, 16 * scale), arm_size, (40 * scale, 48 * scale));

    // copy the leg sides
    copy_rotated_right_flipped(&mut new_skin, (0, 20 * scale), (16 * scale, 12 * scale), (16 * scale, 52 * scale), 4 * scale);
    
    // copy the arm sides
    copy_rotated_right_flipped(&mut new_skin, (40 * scale, 20 * scale), (16 * scale, 12 * scale), (32 * scale, 52 * scale), 4 * scale);

    new_skin
}
