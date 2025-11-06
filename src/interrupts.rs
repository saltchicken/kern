use crate::command;
use crate::print;
use crate::vga_buffer::WRITER;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame}; // We only need print

// Define the offsets for the PIC
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Create a static, mutex-protected instance of the PICs
#[allow(clippy::all)]
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Create the IDT
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Add the handler for the timer interrupt
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        // Set the handler for the keyboard interrupt
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// This function is called by main.rs to load the IDT
pub fn init_idt() {
    IDT.load();
}

// Enum to map PIC interrupt vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // This will be PIC_1_OFFSET + 1 (i.e., 33)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// This is our timer interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Notify the PIC that we are done with the timer interrupt
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// This is our actual keyboard interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

    lazy_static! {
        static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            spin::Mutex::new(Keyboard::new(
                layouts::Us104Key,
                ScancodeSet1,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode('\n') => {
                    // Process the command and print a new prompt
                    command::process_command();
                    print!("\n> ");
                }
                DecodedKey::Unicode('\u{8}') => {
                    // Call our new backspace function
                    command::remove_char();
                    WRITER.lock().backspace();
                }
                DecodedKey::Unicode(character) => {
                    command::add_char(character as u8);
                    print!("{}", character)
                }
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    // Notify the PIC that we are done
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
