#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::global_asm;

use crate::pxe::{get_cached_info, tftp_close, tftp_open, tftp_read, PxeStructure};

mod fail;
mod print;
mod pxe;
mod segmented_ptr;

global_asm!(include_str!("stage_1.s"));
global_asm!(include_str!("stage_2.s"));
global_asm!(include_str!("stage_3.s"));
global_asm!(include_str!("e820.s"));
global_asm!(include_str!("vesa.s"));

// FIXME: The configuration should be extracted from the kernel.
global_asm!(include_str!("vesa_config.s"));

extern "C" {
    static mut kernel_size: usize;
}

#[no_mangle]
pub extern "C" fn download_files() {
    PxeStructure::get().check_signature();

    println!("Querying server address");
    let cached_info = get_cached_info();
    let server_ip_address: [u8; 4] = bytemuck::pod_read_unaligned(&cached_info[20..24]);

    unsafe {
        download_file(server_ip_address, "pxe_bootloader", 0x20_0000 as *mut u8);
        kernel_size = download_file(server_ip_address, "kernel-x86_64", 0x40_0000 as *mut u8);
    }

    // TODO: Stop and Unload the Base Code Stack and UNDI.
    //       This is not technically required, but frees up some memory for the
    //       OS.
}

/// Download a file from a TFTP server to the given address. Returns the size
/// of the downloaded file.
///
/// # Safety
///
/// `base_ptr` must be valid for the entire size of the file.
unsafe fn download_file(server_ip_address: [u8; 4], filename: &str, base_ptr: *mut u8) -> usize {
    const MAX_PACKET_SIZE: u16 = 1024;

    println!("Connecting to {server_ip_address:?} to download {filename}");

    let packet_size = tftp_open(server_ip_address, filename, MAX_PACKET_SIZE);
    let packet_size = usize::from(packet_size);
    println!("Negotiated packet size: {packet_size}");

    let mut packet_buffer = [0u8; MAX_PACKET_SIZE as usize];
    // Truncate the buffer to the negotiated packet size.
    let packet_buffer = &mut packet_buffer[..packet_size];

    let mut len = 0;

    loop {
        // Read from the TFTP connection.
        let buffer = tftp_read(packet_buffer);

        // Copy the downloaded chunk to the destination.
        unsafe {
            core::ptr::copy_nonoverlapping(&buffer[0], base_ptr.add(len), buffer.len());
        }
        len += buffer.len();

        print!("\rRead {len} bytes");

        // The last chunk will be smaller.
        if buffer.len() < packet_size {
            break;
        }
    }

    println!();

    println!("Closing connection");
    tftp_close();

    len
}
