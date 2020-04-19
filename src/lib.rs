#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
use std::thread;
use std::time::Duration;

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

    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    let sleep_times = [0, 100, 500, 200, 5000, 100, 150, 0, 0, 300];

    for i in 0..10 {
        let sleep_time = sleep_times[i];
        // Spin up another thread
        children.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(sleep_time));
            println!("this is thread number {}", i);
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    println!("[main] test returned: {}", x); // 3

    println!("{}", std::fs::read_to_string("sd:/test.txt").unwrap());
    for x in std::fs::read_dir("sd:/atmosphere").unwrap() {
        println!("{:?}", x.unwrap());
    }

    // keep-alive thread
    thread::spawn(||{
        loop {
            println!("Still alive?");
            thread::sleep(Duration::from_secs(3));
            println!("Still alive!");
            thread::sleep(Duration::from_secs(1));
        }
    });
}
