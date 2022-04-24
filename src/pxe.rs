use std::path::Path;

use anyhow::Context;

pub fn create_tftp_folder(
    first_stage_path: &Path,
    bootloader_path: &Path,
    kernel_binary: &Path,
    out_path: &Path,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(out_path)
        .with_context(|| format!("failed to create out dir at {}", out_path.display()))?;

    let to = out_path.join("pxe_first_stage");
    std::fs::copy(first_stage_path, &to).with_context(|| {
        format!(
            "failed to copy first stage from {} to {}",
            first_stage_path.display(),
            to.display()
        )
    })?;

    let to = out_path.join("pxe_bootloader");
    std::fs::copy(bootloader_path, &to).with_context(|| {
        format!(
            "failed to copy bootloader from {} to {}",
            bootloader_path.display(),
            to.display()
        )
    })?;

    let to = out_path.join("kernel-x86_64");
    std::fs::copy(kernel_binary, &to).with_context(|| {
        format!(
            "failed to copy kernel from {} to {}",
            kernel_binary.display(),
            to.display()
        )
    })?;

    Ok(())
}
