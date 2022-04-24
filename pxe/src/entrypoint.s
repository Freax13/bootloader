.section .entrypoint, "awx"
.global _start
.code64

_start:
    # Switch to a bigger stack.
    mov esp, 0x180000

    # Jump to Rust code.
    jmp rust_start