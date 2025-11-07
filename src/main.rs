#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use core::panic::PanicInfo;
mod command;
mod interrupts;
mod vga_buffer;
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // WRITER.lock().clear_screen();
    println!("Welcome{}", "!");
    // Initialize the IDT and PICs
    interrupts::init_idt();

    x86_64::instructions::interrupts::without_interrupts(|| {
        unsafe { interrupts::PICS.lock().initialize() };
    });

    // Enable hardware interrupts
    x86_64::instructions::interrupts::enable();

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
