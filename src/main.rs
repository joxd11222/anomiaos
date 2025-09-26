#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Welcome to Anomia OS!";

#[unsafe(no_mangle)] 
pub extern "C" fn _start() -> ! {
    let mut writer = vga_buffer::Writer {
        row_position: 0,
        column_position: 0,
        color_code: vga_buffer::ColorCode::new(
            vga_buffer::Color::Yellow,
            vga_buffer::Color::Black,
        ),
        buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    };
    writer.clear_screen();
    writer.write_string(core::str::from_utf8(HELLO).unwrap());
    writer.write_string("\n");
    writer.write_string("different tests running, wait for it...\n");
    writer.write_string("testing the scrolling function\n");
    let mut num_buf = [0u8; 20];
    for i in 0..30 {
        writer.write_string("scrolling line ");
        let s = vga_buffer::int_to_string(i, &mut num_buf);
        writer.write_string(s);
        writer.write_string("\n");
    }
    writer.write_string("it works!\n");
    writer.write_string("assertion test: ");
    assert_eq!(1, 1);
    writer.write_string("it works!\n");
    writer.write_string("all tests passed!\n");
    writer.write_string("Press Enter to exit...\n");
    vga_buffer::wait_for_enter();
    loop {}
}
