use std::{
    path::{Path, PathBuf},
    process::Command,
};

const BOOTLOADER_X86_64_UEFI_VERSION: &str = "0.1.0-alpha.0";
const BOOTLOADER_X86_64_BIOS_BOOT_SECTOR_VERSION: &str = "0.1.0-alpha.0";
const BOOTLOADER_X86_64_BIOS_SECOND_STAGE_VERSION: &str = "0.1.0-alpha.0";
const BOOTLOADER_X86_64_PXE_FIRST_STAGE_VERSION: &str = "0.1.0-alpha.0";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let uefi_path = build_uefi_bootloader(&out_dir);
    println!(
        "cargo:rustc-env=UEFI_BOOTLOADER_PATH={}",
        uefi_path.display()
    );

    let bios_boot_sector_path = build_bios_boot_sector(&out_dir);
    println!(
        "cargo:rustc-env=BIOS_BOOT_SECTOR_PATH={}",
        bios_boot_sector_path.display()
    );
    let bios_second_stage_path = build_bios_second_stage(&out_dir);
    println!(
        "cargo:rustc-env=BIOS_SECOND_STAGE_PATH={}",
        bios_second_stage_path.display()
    );

    let pxe_first_stage_path = build_pxe_first_stage(&out_dir);
    println!(
        "cargo:rustc-env=PXE_FIRST_STAGE_PATH={}",
        pxe_first_stage_path.display()
    );
    let pxe_bootloader_path = build_pxe_bootloader(&out_dir);
    println!(
        "cargo:rustc-env=PXE_BOOTLOADER_PATH={}",
        pxe_bootloader_path.display()
    );
}

fn build_uefi_bootloader(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-uefi");
    if Path::new("uefi").exists() {
        // local build
        cmd.arg("--path").arg("uefi");
    } else {
        cmd.arg("--version").arg(BOOTLOADER_X86_64_UEFI_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("x86_64-unknown-uefi");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    let status = cmd
        .status()
        .expect("failed to run cargo install for uefi bootloader");
    if status.success() {
        let path = out_dir.join("bin").join("bootloader-x86_64-uefi.efi");
        assert!(
            path.exists(),
            "uefi bootloader executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build uefi bootloader");
    }
}

fn build_bios_boot_sector(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-bios-boot-sector");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bios")
        .join("boot_sector");
    if local_path.exists() {
        // local build
        cmd.arg("--path").arg(&local_path);
    } else {
        cmd.arg("--version")
            .arg(BOOTLOADER_X86_64_BIOS_BOOT_SECTOR_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("x86-16bit.json");
    cmd.arg("--profile").arg("first-stage");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .expect("failed to run cargo install for bios boot sector");
    let elf_path = if status.success() {
        let path = out_dir
            .join("bin")
            .join("bootloader-x86_64-bios-boot-sector");
        assert!(
            path.exists(),
            "bios boot sector executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build bios boot sector");
    };
    convert_elf_to_bin(elf_path)
}
fn build_bios_second_stage(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install")
        .arg("bootloader-x86_64-bios-second-stage");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bios")
        .join("second_stage");
    if local_path.exists() {
        // local build
        cmd.arg("--path").arg(&local_path);
    } else {
        cmd.arg("--version")
            .arg(BOOTLOADER_X86_64_BIOS_SECOND_STAGE_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("x86-16bit-second-stage.json");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .expect("failed to run cargo install for bios second stage");
    let elf_path = if status.success() {
        let path = out_dir
            .join("bin")
            .join("bootloader-x86_64-bios-second-stage");
        assert!(
            path.exists(),
            "bios second stage executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build bios second stage");
    };
    convert_elf_to_bin(elf_path)
}

fn build_pxe_first_stage(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-pxe-first-stage");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pxe")
        .join("first_stage");
    if local_path.exists() {
        // local build
        cmd.arg("--path").arg(&local_path);
    } else {
        cmd.arg("--version")
            .arg(BOOTLOADER_X86_64_PXE_FIRST_STAGE_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("x86-16bit.json");
    cmd.arg("--profile").arg("pxe-first-stage");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .expect("failed to run cargo install for pxe first stage");
    let elf_path = if status.success() {
        let path = out_dir
            .join("bin")
            .join("bootloader-x86_64-pxe-first-stage");
        assert!(
            path.exists(),
            "pxe first stage executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build pxe first stage");
    };

    let bin = convert_elf_to_bin(elf_path);

    // Make sure that the first stage doesn't exceed the recommended size.
    let metadata = std::fs::metadata(&bin).unwrap();
    assert!(
        metadata.len() <= 32768,
        "first pxe stage is too big: {} bytes",
        metadata.len()
    );

    bin
}

fn build_pxe_bootloader(out_dir: &Path) -> PathBuf {
    let bootloader_path = std::env::var("CARGO_BIN_FILE_BOOTLOADER_X86_64_PXE")
        .expect("binary dependency should set `CARGO_BIN_FILE_BOOTLOADER_X86_64_PXE`");
    let bootloader_path =
        Path::new("/home/freax13/Documents/code/rust/bootloader/bootloader-x86_64-pxe");
    let new_bootloader_path = out_dir.join("pxe-bootloader");
    std::fs::copy(bootloader_path, &new_bootloader_path).unwrap();
    convert_elf_to_bin(new_bootloader_path)
}

fn convert_elf_to_bin(elf_path: PathBuf) -> PathBuf {
    let flat_binary_path = elf_path.with_extension("bin");

    let llvm_tools = llvm_tools::LlvmTools::new().expect("failed to get llvm tools");
    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("LlvmObjcopyNotFound");

    // convert first stage to binary
    let mut cmd = Command::new(objcopy);
    cmd.arg("-I").arg("elf64-x86-64");
    cmd.arg("-O").arg("binary");
    cmd.arg("--binary-architecture=i386:x86-64");
    cmd.arg(&elf_path);
    cmd.arg(&flat_binary_path);
    let output = cmd
        .output()
        .expect("failed to execute llvm-objcopy command");
    if !output.status.success() {
        panic!(
            "objcopy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    flat_binary_path
}
