macro_rules! costume_file {
    ($e:expr) => {
        [
            smash::hash40(concat!("fighter/pickel/model/body/c00/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c01/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c02/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c03/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c04/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c05/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c06/", $e)),
            smash::hash40(concat!("fighter/pickel/model/body/c07/", $e)),
        ]
    };
}

pub static STEVE_NUTEXB_FILES: [u64; 8] = costume_file!("def_pickel_001_col.nutexb");

pub static STEVE_NUMSHB_FILES: [u64; 8] = costume_file!("model.numshb");

pub static STEVE_NUMATB_FILES: [u64; 8] = costume_file!("model.numatb");
pub static STEVE_LIGHT_MODEL_FILES: [u64; 8] = costume_file!("light_model.numatb");
pub static STEVE_DARK_MODEL_FILES: [u64; 8] = costume_file!("dark_model.numatb");
pub static STEVE_METAMON_MAT_FILES: [u64; 8] = costume_file!("metamon_model.numatb");
pub static STEVE_EMI_FILES: [u64; 8] = costume_file!("def_pickel_001_emi.nutexb");
pub static STEVE_MESHEXT_FILES: [u64; 8] = costume_file!("model.numshexb");
pub static STEVE_NUSRCMDLB_FILES: [u64; 8] = costume_file!("model.nusrcmdlb");
pub static STEVE_MODL_FILES: [u64; 8] = costume_file!("model.numdlb");
pub static STEVE_STOCK_ICONS: [u64; 8] = [
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_00.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_01.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_02.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_03.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_04.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_05.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_06.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_2/chara_2_pickel_07.bntx"),
];


//pub static STEVE_CHARA_1: [u64; 8] = [
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_00.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_01.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_02.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_03.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_04.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_05.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_06.bntx"),
//    smash::hash40("ui/replace_patch/chara/chara_1/chara_1_pickel_07.bntx"),
//];

pub static STEVE_CHARA_3: [u64; 8] = [
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_00.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_01.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_02.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_03.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_04.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_05.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_06.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_3/chara_3_pickel_07.bntx"),
];

pub static STEVE_CHARA_4: [u64; 8] = [
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_00.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_01.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_02.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_03.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_04.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_05.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_06.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_4/chara_4_pickel_07.bntx"),
];

pub static STEVE_CHARA_6: [u64; 8] = [
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_00.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_01.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_02.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_03.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_04.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_05.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_06.bntx"),
    smash::hash40("ui/replace_patch/chara/chara_6/chara_6_pickel_07.bntx"),
];

