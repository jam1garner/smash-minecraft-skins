use skyline::libc::c_void;
use skyline::libc::free;

#[repr(C)]
pub struct ShowKeyboardArg {
    pub keyboard_config: [u8; 0x4D0],
    pub work_buffer: *const c_void,
    pub work_buffer_size: usize,
    pub text_buffer: *const c_void,
    pub text_buffer_size: usize,
    pub custom_dictionary_buffer: *const c_void,
    pub custom_dictionary_buffer_size: usize,
}

#[repr(transparent)]
pub struct SwkbdString(Box<[u16]>);

impl SwkbdString {
    fn that_big_size() -> Self {
        let x: Box<[u16; 1002]> = unsafe { Box::new_zeroed().assume_init() };
        SwkbdString(x)
    }
}

impl From<SwkbdString> for String {
    fn from(s: SwkbdString) -> String {
        let end = s.0.iter().position(|c| *c == 0).unwrap_or_else(|| s.0.len());
        String::from_utf16_lossy(&s.0[..end])
    }
}

extern "C" {
    #[link_name = "_ZN2nn5swkbd17MakePresetDefaultEPNS0_14KeyboardConfigE"]
    fn make_preset_default(x: *mut [u8; 0x4d0]);
    
    #[link_name = "_ZN2nn5swkbd12ShowKeyboardEPNS0_6StringERKNS0_15ShowKeyboardArgE"]
    fn show_keyboard(string: *const SwkbdString, arg: *const ShowKeyboardArg) -> u32;

    //#[link_name = "_ZN2nn5swkbd17SetHeaderTextUtf8EPNS0_14KeyboardConfigEPKc"]
    #[link_name = "_ZN2nn5swkbd13SetHeaderTextEPNS0_14KeyboardConfigEPKDs"]
    fn set_header_text(x: *const [u8; 0x4d0], text: *const u16);
}

impl ShowKeyboardArg {
    pub fn new() -> Box<Self> {
        let mut arg: Box<Self> = unsafe { Box::new_zeroed().assume_init() };

        unsafe {
            make_preset_default(&mut arg.keyboard_config);
        }
        // max length
        arg.keyboard_config[0x3ac] = 20;
        // mode
        arg.keyboard_config[0x3b8] = 0;
        // cancel
        arg.keyboard_config[0x3bc] = 0;



        let work_buffer_size = 0xd000;
        let work_buffer: Box<[u8; 0xd000]> = unsafe { Box::new_zeroed().assume_init() };
        let work_buffer = Box::leak(work_buffer) as *const _ as *const c_void;

        arg.work_buffer = work_buffer;
        arg.work_buffer_size = work_buffer_size;
        
        arg
    }

    pub fn header_text(&mut self, s: &str) -> &mut Self {
        let x: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            set_header_text(&self.keyboard_config, x.as_ptr() as _);
        }

        std::mem::drop(x);

        self
    }

    pub fn show(&self) -> Option<String> {
        let string = SwkbdString::that_big_size();
        if unsafe { show_keyboard(&string, self) } == 0x29f {
            None
        } else {
            Some(string.into())
        }
    }
}

impl Drop for ShowKeyboardArg {
    fn drop(&mut self) {
        if !self.work_buffer.is_null() {
            unsafe {
                free(self.work_buffer);
            }
        }
    }
}
