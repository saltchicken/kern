use crate::vga_buffer::WRITER;
use crate::{print, println};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

// Set the max command length (e.g., one line)
const COMMAND_BUFFER_SIZE: usize = 80;

struct CommandBuffer {
    buffer: [u8; COMMAND_BUFFER_SIZE],
    len: usize,
}

lazy_static! {
    /// A global, locked buffer for storing the user's current command.
    static ref COMMAND_BUFFER: Mutex<CommandBuffer> = Mutex::new(CommandBuffer {
        buffer: [0; COMMAND_BUFFER_SIZE],
        len: 0,
    });
}

/// Adds a character to the global command buffer.
pub fn add_char(c: u8) {
    let mut buffer = COMMAND_BUFFER.lock();
    if buffer.len < COMMAND_BUFFER_SIZE {
        let len = buffer.len;
        buffer.buffer[len] = c;
        buffer.len += 1;
    }
}

/// Removes the last character from the global command buffer.
pub fn remove_char() {
    let mut buffer = COMMAND_BUFFER.lock();
    if buffer.len > 0 {
        buffer.len -= 1;
    }
}

/// Processes the current command in the buffer.
pub fn process_command() {
    let mut buffer = COMMAND_BUFFER.lock();

    // Get a slice of the command
    let command_slice = &buffer.buffer[..buffer.len];

    // Convert to &str and process
    match core::str::from_utf8(command_slice) {
        Ok("clear") => {
            interrupts::without_interrupts(|| {
                WRITER.lock().clear_screen();
            });
        }
        Ok(cmd) => {
            // Print an "unknown command" message
            println!("\nUnknown command: \"{}\"", cmd);
        }
        Err(_) => {
            println!("\nError: Non-UTF8 command");
        }
    }

    // Reset buffer for the next command
    buffer.len = 0;
}
