ENTRY(_start)

SECTIONS {
    . = 0x200000;
    .entrypoint :
    {
        *(.entrypoint .entrypoint.*)
    }.text :
    {
        *(.text .text.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
        *(.bss .bss.*)
    }
}
