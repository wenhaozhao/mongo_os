#![no_std]
#![no_main]

use mongo_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Welcome to MongoOS");
    mongo_os::init();
    mongo_os::halt_loop()
}

