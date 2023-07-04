use x86_64::structures::idt::InterruptStackFrame;
use crate::idt::{InterruptIndex, PICS};
use crate::print;

pub extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame){
    //todo
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}