.section .boot, "awx"
.global _start
.code16

# This stage initializes the stack, enables the A20 line

_start:
    # Pop the pointer to !PXE
    add sp, 4
    pop ecx

    # zero segment registers
    xor eax, eax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    # Save the pointer to !PXE
    mov pxe_structure_pointer, ecx

    # Fix the stack
    lea sp, _stack_end

    # clear the direction flag (e.g. go forward in memory when using
    # instructions like lodsb)
    cld

enable_a20:
    # enable A20-Line via IO-Port 92, might not work on all motherboards
    in al, 0x92
    test al, 2
    jnz enable_a20_after
    or al, 2
    and al, 0xFE
    out 0x92, al
enable_a20_after:
    call enter_unreal_mode
    jmp rust

enter_unreal_mode:
enter_protected_mode:
    # clear interrupts
    cli
    push ds
    push es

    lgdt [gdt32info]

    mov eax, cr0
    or al, 1    # set protected mode bit
    mov cr0, eax

    jmp protected_mode                # tell 386/486 to not crash

protected_mode:
    mov bx, 0x10
    mov ds, bx # set data segment
    mov es, bx # set extra segment

    and al, 0xfe    # clear protected mode bit
    mov cr0, eax

unreal_mode:
    pop es # get back old extra segment
    pop ds # get back old data segment
    sti

    # back to real mode, but internal data segment register is still loaded
    # with gdt segment -> we can access the full 4GiB of memory

    # mov bx, 0x0f01         # attrib/char of smiley
    # mov eax, 0xb8f00       # note 32 bit offset
    # mov word ptr ds:[eax], bx

    ret

rust:
    call download_files

jump_to_second_stage:
    mov eax, offset stage_2
    jmp eax

spin:
    hlt
    jmp spin


gdt32info:
   .word gdt32_end - gdt32 - 1  # last byte in table
   .word gdt32                  # start of table

gdt32:
    # entry 0 is always unused
    .quad 0
codedesc:
    .byte 0xff
    .byte 0xff
    .byte 0
    .byte 0
    .byte 0
    .byte 0x9a
    .byte 0xcf
    .byte 0
datadesc:
    .byte 0xff
    .byte 0xff
    .byte 0
    .byte 0
    .byte 0
    .byte 0x92
    .byte 0xcf
    .byte 0
gdt32_end:

.global pxe_structure_pointer
pxe_structure_pointer:
    .double 0
.global kernel_size
kernel_size:
    .double 0

# print a string and a newline
# IN
#   si: points at zero-terminated String
# CLOBBER
#   ax
real_mode_println:
    call real_mode_print
    mov al, 13 # \r
    call real_mode_print_char
    mov al, 10 # \n
    jmp real_mode_print_char

# print a string
# IN
#   si: points at zero-terminated String
# CLOBBER
#   ax
real_mode_print:
    cld
real_mode_print_loop:
    # note: if direction flag is set (via std)
    # this will DECREMENT the ptr, effectively
    # reading/printing in reverse.
    lodsb al, BYTE PTR [si]
    test al, al
    jz real_mode_print_done
    call real_mode_print_char
    jmp real_mode_print_loop
real_mode_print_done:
    ret

# print a character
# IN
#   al: character to print
# CLOBBER
#   ah
real_mode_print_char:
    mov ah, 0x0e
    int 0x10
    ret