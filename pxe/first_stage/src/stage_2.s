.section .boot, "awx"
.code16

# This stage sets the target operating mode, loads the kernel from disk,
# creates an e820 memory map, enters protected mode, and jumps to the
# third stage.

second_stage_start_str: .asciz "Booting (second stage)..."
kernel_load_failed_str: .asciz "Failed to load kernel from disk"

kernel_load_failed:
    mov si, offset kernel_load_failed_str
    call real_mode_println
kernel_load_failed_spin:
    jmp kernel_load_failed_spin

stage_2:
    mov si, offset second_stage_start_str
    call real_mode_println

set_target_operating_mode:
    # Some BIOSs assume the processor will only operate in Legacy Mode. We change the Target
    # Operating Mode to "Long Mode Target Only", so the firmware expects each CPU to enter Long Mode
    # once and then stay in it. This allows the firmware to enable mode-specifc optimizations.
    # We save the flags, because CF is set if the callback is not supported (in which case, this is
    # a NOP)
    pushf
    mov ax, 0xec00
    mov bl, 0x2
    int 0x15
    popf

create_memory_map:
    lea di, es:[_memory_map]
    call do_e820

video_mode_config:
    call vesa

enter_protected_mode_again:
    cli
    lgdt [gdt32info]
    mov eax, cr0
    or al, 1    # set protected mode bit
    mov cr0, eax

    push 0x8
    mov eax, offset stage_3
    push eax
    retf

spin32:
    jmp spin32



# print a string and a newline
# IN
#   esi: points at zero-terminated String
vga_println:
    push eax
    push ebx
    push ecx
    push edx

    call vga_print

    # newline
    mov edx, 0
    mov eax, vga_position
    mov ecx, 80 * 2
    div ecx
    add eax, 1
    mul ecx
    mov vga_position, eax

    pop edx
    pop ecx
    pop ebx
    pop eax

    ret

# print a string
# IN
#   esi: points at zero-terminated String
# CLOBBER
#   ah, ebx
vga_print:
    cld
vga_print_loop:
    # note: if direction flag is set (via std)
    # this will DECREMENT the ptr, effectively
    # reading/printing in reverse.
    lodsb al, BYTE PTR [esi]
    test al, al
    jz vga_print_done
    call vga_print_char
    jmp vga_print_loop
vga_print_done:
    ret


# print a character
# IN
#   al: character to print
# CLOBBER
#   ah, ebx
vga_print_char:
    mov ebx, vga_position
    mov ah, 0x0f
    mov [ebx + 0xa0000], ax

    add ebx, 2
    mov [vga_position], ebx

    ret

vga_position:
    .double 0

    