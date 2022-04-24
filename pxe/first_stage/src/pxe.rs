use core::{arch::asm, ffi::c_void};

use crate::segmented_ptr::SegmentedPointer;

const PXENV_STATUS_SUCCESS: u16 = 0;
const PXENV_STATUS_FAILURE: u16 = 1;

#[repr(C)]
pub struct PxeStructure {
    signature: [u8; 4],
    struct_length: u8,
    struct_chksum: u8,
    _reserved: u8,
    undi_rom_id: SegmentedPointer<'static, UndiRomId>,
    base_rom_id: SegmentedPointer<'static, BaseRomId>,
    entry_point_sp: SegmentedPointer<'static, EntryPoint>,
    entry_point_esp: SegmentedPointer<'static, c_void>,
}

impl PxeStructure {
    pub fn get() -> &'static Self {
        extern "C" {
            static pxe_structure_pointer: SegmentedPointer<'static, PxeStructure>;
        }
        unsafe { &*pxe_structure_pointer }
    }

    pub fn check_signature(&self) {
        assert_eq!(self.signature, *b"!PXE", "Invalid signature");
    }
}

// FIXME: Fill in
struct UndiRomId;

// FIXME: Fill in
struct BaseRomId;

pub struct EntryPoint(());

unsafe fn execute(opcode: u16, parameter_structure: &mut [u8]) -> Result<(), u16> {
    // Make sure there's space for the status.
    assert!(parameter_structure.len() >= 2);

    let ptr = SegmentedPointer::from_mut(&mut parameter_structure[0]);
    let entry_point = &PxeStructure::get().entry_point_sp;
    let exit_code: u16;

    unsafe {
        asm!(
            // LLVM forbids us to clobber those registers:
            // Save ebp and esi on the stack.
            "push ebp",
            "push esi",

            "push ax",
            "push bx",
            "push cx",
            "lcall [edx]",
            "add esp, 6",

            // Restore ebp and esi.
            "pop esi",
            "pop ebp",

            // Some Bios implementations change the segment descriptors back to
            // regular Real mode. Reenter Unreal mode.
            "push ax",
            "call enter_unreal_mode",
            "pop ax",

            inout("ax") ptr.segment() => exit_code,
            inout("bx") ptr.offset() => _,
            inout("cx") opcode => _,
            inout("edx") entry_point => _,
            out("edi") _,
        );
    }

    if exit_code == 0 {
        Ok(())
    } else {
        let status = bytemuck::pod_read_unaligned(&parameter_structure[0..2]);
        Err(status)
    }
}

pub fn get_cached_info() -> &'static [u8] {
    const PXENV_GET_CACHED_INFO: u16 = 0x71;

    let mut parameter_structure = [0; 2 + 2 + 2 + 4 + 2];
    parameter_structure[2..4].copy_from_slice(&3u16.to_le_bytes()); // Packet-Type: 3

    let res = unsafe { execute(PXENV_GET_CACHED_INFO, &mut parameter_structure) };
    if let Err(status) = res {
        panic!("Failed to get cached info: {status:#06x}");
    }

    let buffer_size: u16 = bytemuck::pod_read_unaligned(&parameter_structure[4..6]);
    let offset: u16 = bytemuck::pod_read_unaligned(&parameter_structure[6..8]);
    let segment: u16 = bytemuck::pod_read_unaligned(&parameter_structure[8..10]);

    let addr = (u32::from(segment) << 4) + u32::from(offset);
    let ptr = addr as *const u8;
    let len = usize::from(buffer_size);

    unsafe { core::slice::from_raw_parts(ptr, len) }
}

pub fn tftp_open(server_ip_address: [u8; 4], filename: &str, packet_size: u16) -> u16 {
    const PXENV_TFTP_OPEN: u16 = 0x20;

    assert!(filename.len() < 128);
    assert!(packet_size >= 512);

    let mut parameter_structure = [0; 2 + 4 + 4 + 128 + 2 + 2];
    parameter_structure[2..6].copy_from_slice(&server_ip_address); // ServerIPAddress
    parameter_structure[10..][..filename.len()].copy_from_slice(filename.as_bytes()); // FileName
    parameter_structure[138..140].copy_from_slice(&69u16.to_be_bytes()); // TFTPPort
    parameter_structure[140..142].copy_from_slice(&packet_size.to_le_bytes()); // PacketSize

    let res = unsafe { execute(PXENV_TFTP_OPEN, &mut parameter_structure) };
    if let Err(status) = res {
        panic!("Failed to open a TFTP connection: {status:#06x}");
    }

    let negotiated_packet_size: u16 = bytemuck::pod_read_unaligned(&parameter_structure[140..142]);
    negotiated_packet_size
}

pub fn tftp_close() {
    const PXENV_TFTP_CLOSE: u16 = 0x21;

    let mut parameter_structure = [0; 2];

    let res = unsafe { execute(PXENV_TFTP_CLOSE, &mut parameter_structure) };
    if let Err(status) = res {
        panic!("Failed to read from TFTP connection: {status:#06x}");
    }
}

pub fn tftp_read<'a>(buffer: &'a mut [u8]) -> &'a mut [u8] {
    const PXENV_TFTP_READ: u16 = 0x22;

    let ptr = SegmentedPointer::from_mut(&mut buffer[0]);

    let mut parameter_structure = [0; 2 + 2 + 2 + 4];
    parameter_structure[6..8].copy_from_slice(&ptr.offset().to_le_bytes());
    parameter_structure[8..10].copy_from_slice(&ptr.segment().to_le_bytes());

    let res = unsafe { execute(PXENV_TFTP_READ, &mut parameter_structure) };
    if let Err(status) = res {
        panic!("Failed to close the TFTP connection: {status:#06x}");
    }

    let buffer_size: u16 = bytemuck::pod_read_unaligned(&parameter_structure[4..6]);
    let len = usize::from(buffer_size);
    &mut buffer[..len]
}
