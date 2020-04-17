#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};

extern "C" fn test() -> u32 {
    2
}

//#[hook(offset = 0x12345)]
#[hook(replace = test)]
fn test_replacement() -> u32 {

    let original_test = original!();

    let val = original_test();

    println!("[override] original value: {}", val); // 2

    val + 1
}

#[skyline::main(name = "skyline_rs_template")]
pub fn main() {
    println!("Hello from Skyline Rust Plugin!");

    install_hook!(test_replacement);

    let x = test();

    println!("[main] test returned: {}", x); // 3

    println!("{}", std::fs::read_to_string("sd:/test.txt").unwrap());
    for x in std::fs::read_dir("sd:/atmosphere").unwrap() {
        println!("{:?}", x.unwrap());
    }
}
