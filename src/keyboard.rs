use x86_64::structures::idt::InterruptStackFrame;

use crate::idt::{InterruptIndex, PICS};
use crate::print;

const PS2_IO_PORT_ADDR: u16 = 0x60;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame) {
    //todo
    use lazy_static::lazy_static;
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    use pc_keyboard::{HandleControl, Keyboard, DecodedKey};
lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<pc_keyboard::layouts::Us104Key, pc_keyboard::ScancodeSet1>>={
        Mutex::new(Keyboard::new(
            pc_keyboard::ScancodeSet1::new(),
            pc_keyboard::layouts::Us104Key,
                HandleControl::Ignore
        ))
    };
}
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(PS2_IO_PORT_ADDR);
    let scan_code: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                _ => {
                    //DecodedKey::RawKey(key) => print!("{:?}", key)
                },
            }
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}