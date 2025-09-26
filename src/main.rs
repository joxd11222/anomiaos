#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = vga_buffer::Writer {
        row_position: 0,
        column_position: 0,
        color_code: vga_buffer::ColorCode::new(
            vga_buffer::Color::Red,
            vga_buffer::Color::Black,
        ),
        buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    };
    writer.clear_screen();
    writer.write_string("!!! PANIC !!!\n");
    if let Some(location) = info.location() {
        writer.write_string("panic at ");
        writer.write_string(location.file());
        writer.write_string(":");
        let mut buf = [0u8; 20];
        writer.write_string(&vga_buffer::int_to_string(location.line() as usize, &mut buf));
        writer.write_string("\n");
    } else {
        writer.write_string("panic location unknown.\n");
    }
    let msg = info.message();
    writer.write_string("panic message: ");
    writer.write_string(msg.as_str().unwrap_or("no know message bruv"));
    writer.write_string("\n");
    loop {}
}

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
    writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
    writer.write_string("==== WELCOME TO ANOMIA OS ====\n");
    writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
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
    writer.write_string("color test:\n");
    vga_buffer::color_test();
    writer.write_string("it works!\n");
    writer.write_string("ascii test:\n");
    vga_buffer::ascii_test();
    writer.write_string("it works!\n");
    writer.write_string("math test:\n");
    vga_buffer::math_test();
    writer.write_string("it works!\n");
    writer.write_string("keyboard test:\n");
    vga_buffer::keyboard_test();
    writer.write_string("it works!\n");
    writer.write_string("panic test:\n");
    vga_buffer::panic_test();
    writer.write_string("all tests passed!\n");
    writer.write_string("Press Enter to exit...\n");
    vga_buffer::wait_for_enter();
    loop {}
}
