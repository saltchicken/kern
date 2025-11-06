#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;
// ‼️ Import the WRITER static ‼️
use crate::vga_buffer::WRITER;
mod command;
mod interrupts;
mod vga_buffer; // ‼️ ADD THIS MODULE ‼️
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // ‼️ Add this line to clear the screen first ‼️
    WRITER.lock().clear_screen();
    println!("Hello World{}", "!");
    // Initialize the IDT and PICs
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    // Enable hardware interrupts
    x86_64::instructions::interrupts::enable();
    println!("Keyboard input is enabled:");

    // ‼️ REMOVED the \n from here ‼️
    print!("> ");

    loop {
        x86_64::instructions::hlt();
    }
}
/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
