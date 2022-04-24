set -e
mkdir -p target/tftp
RUSTFLAGS="-C opt-level=3 -C relocation-model=static -C codegen-units=1 -C lto=thin -C embed-bitcode=yes" cargo build --release -p bootloader-x86_64-pxe-first-stage --target x86-16bit.json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
RUSTFLAGS="-C opt-level=3 -C relocation-model=static -C codegen-units=1 -C lto=fat -C embed-bitcode=yes" cargo build -p bootloader-x86_64-pxe --target x86_64-unknown-none
llvm-objcopy-12 -I elf64-x86-64 -O binary target/x86-16bit/release/bootloader-x86_64-pxe-first-stage target/tftp/pxe_first_stage
llvm-objcopy-12 -I elf64-x86-64 -O binary target/x86_64-unknown-none/debug/bootloader-x86_64-pxe target/tftp/pxe_bootloader
cargo build -p test_kernel_default_settings --target x86_64-unknown-none
cp target/x86_64-unknown-none/debug/basic_boot target/tftp/kernel-x86_64
cp /home/freax13/Documents/code/rust/blocks/target/x86_64-blocks/debug/blocks target/tftp/kernel-x86_64
qemu-system-x86_64 -netdev user,id=net0,net=192.168.17.0/24,tftp=target/tftp/,bootfile=pxe_first_stage -device virtio-net-pci,netdev=net0 -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 # -S -s
# /home/freax13/Documents/code/qemu/build/qemu-system-x86_64 -enable-kvm -netdev user,id=net0,net=192.168.17.0/24,tftp=target/tftp/,bootfile=pxe_first_stage -device virtio-net-pci,netdev=net0 -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 # -S -s