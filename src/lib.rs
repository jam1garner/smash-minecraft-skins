#![feature(proc_macro_hygiene, new_uninit)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

use parking_lot::Mutex;
use image::{Pixel, DynamicImage};

use skyline::hooks::InlineCtx;
use smash::lib::lua_const::FIGHTER_KIND_PICKEL;

mod keyboard;
mod skin_menu;
mod modern_skin;
mod minecraft_api;
mod stock_generation;

use modern_skin::convert_to_modern_skin;

lazy_static::lazy_static! {
    static ref SKINS: Mutex<skin_menu::Skins> = Mutex::new(
        skin_menu::Skins::from_cache().unwrap_or_default()
    );
}

static STEVE_NUTEXB_FILES: [u64; 8] = [
    smash::hash40("fighter/pickel/model/body/c00/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c01/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c02/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c03/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c04/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c05/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c06/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c07/def_pickel_001_col.nutexb"),
];

static STEVE_STOCK_ICONS: [u64; 8] = [
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_00.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_01.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_02.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_03.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_04.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_05.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_06.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_07.bntx"),
];

static SELECTED_SKINS: [Mutex<Option<PathBuf>>; 8] = [
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
];

static LAST_SELECTED: AtomicUsize = AtomicUsize::new(0xFF);

extern "C" {
    #[link_name = "_ZN2nn5prepo10PlayReport3AddEPKcS3_"]
    fn prepo_add_play_report(a: u64, b: u64, c: u64) -> u64;
}

#[skyline::hook(replace = prepo_add_play_report)]
fn prepo_add_play_report_hook(a: u64, b: u64, c: u64) -> u64 {
    LAST_SELECTED.store(0xFF, Ordering::SeqCst);

    original!()(a, b, c)
}

type ArcCallback = extern "C" fn(u64, *mut u8, usize) -> bool;

extern "C" {
    fn subscribe_callback_with_size(hash: u64, filesize: u32, extension: *const u8, extension_len: usize, callback: ArcCallback);
}

const MAX_HEIGHT: usize = 1024;
const MAX_WIDTH: usize = 1024;
const MAX_DATA_SIZE: usize = MAX_HEIGHT * MAX_WIDTH * 4;
const MAX_FILE_SIZE: usize = MAX_DATA_SIZE + 0xb0;

extern "C" fn steve_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_NUTEXB_FILES.iter().position(|&x| x == hash) {
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();

        let skin_data = if let Some(path) = skin_path {
            image::load_from_memory(&fs::read(path).unwrap()).unwrap()
        } else {
            return false
        };

        let mut skin_data = skin_data.to_rgba();

        let (width, height) = skin_data.dimensions();
        if width == height * 2 {
            skin_data = convert_to_modern_skin(&skin_data);
        }

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
    
        //skin_data.save("sd:/test.png");

        let real_size = (skin_data.height() as usize * skin_data.width() as usize * 4) + 0xb0;

        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);
        nutexb::writer::write_nutexb("steve_minecraft???", &DynamicImage::ImageRgba8(skin_data), &mut writer).unwrap();

        let data_out = writer.into_inner();

        if real_size != MAX_FILE_SIZE {
            let start_of_header = real_size - 0xb0;

            let (from, to) = data_out.split_at_mut(MAX_DATA_SIZE);
            to.copy_from_slice(&from[start_of_header..real_size]);
        }

        true
    } else {
        false
    }
}

extern "C" fn steve_stock_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_STOCK_ICONS.iter().position(|&x| x == hash) {
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();

        let skin_data = if let Some(path) = skin_path {
            image::load_from_memory(&fs::read(path).unwrap()).unwrap()
        } else {
            return false
        };

        let skin = skin_data.to_rgba();
        let stock_icon = stock_generation::gen_stock_image(&skin);

        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        bntx::BntxFile::from_image(DynamicImage::ImageRgba8(skin), "steve")
            .write(&mut writer)
            .unwrap();

        true
    } else {
        false
    }
}

#[derive(Debug)]
struct UnkPtr1 {
    ptrs: [&'static u64; 7],
}

#[derive(Debug)]
struct UnkPtr2 {
    bunch_bytes: [u8; 0x20],
    bunch_bytes2: [u8; 0x20]
}

#[derive(Debug)]
#[repr(C)]
pub struct FighterInfo {
    unk_ptr1: &'static UnkPtr1,
    unk_ptr2: &'static UnkPtr2,
    unk1: [u8; 0x20],
    unk2: [u8; 0x20],
    unk3: [u8; 0x8],
    fighter_id: u8,
    unk4: [u8;0xB],
    fighter_slot: u8,
}

#[skyline::hook(offset = 0x661acc, inline)]
fn css_fighter_selected(ctx: &InlineCtx) {
    let infos = unsafe { &*(ctx.registers[0].bindgen_union_field as *const FighterInfo) };

    let is_steve = *FIGHTER_KIND_PICKEL == infos.fighter_id as i32;

    if is_steve {
        let slot = infos.fighter_slot as usize;
        
        *SELECTED_SKINS[slot].lock() = SKINS.lock().get_skin_path();
    }
}

const MAX_STOCK_ICON_SIZE: u32 = 0x9c68;

#[skyline::main(name = "minecraft_skins")]
pub fn main() {
    skyline::install_hooks!(prepo_add_play_report_hook, css_fighter_selected);

    unsafe {
        for hash in &STEVE_NUTEXB_FILES {
            subscribe_callback_with_size(*hash, MAX_FILE_SIZE as _, "nutexb".as_ptr(), "nutexb".len(), steve_callback);
        }

        /*for hash in &STEVE_STOCK_ICONS {
            subscribe_callback_with_size(*hash, MAX_STOCK_ICON_SIZE as _, "bntx".as_ptr(), "bntx".len(), steve_stock_callback);
        }*/
    }
}
