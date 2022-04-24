ENTRY(_start)

SECTIONS {
    . = 0x7c00;
    .boot :
    {
        *(.boot .boot.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }

    . = ALIGN(4);
    _stack_start = .;
    . = _stack_start + 0x600;
    _stack_end = .;
    
    _memory_map = .;
    . += 0x1000;

    /* page tables */
    . = ALIGN(0x1000);
    __page_table_start = .;
    _p4 = .;
    . += 0x1000;
    _p3 = .;
    . += 0x1000;
    _p2 = .;
    . += 0x1000;
    _p1 = .;
    . += 0x1000;
    __page_table_end = .;
}

