use anyhow::Result;
use log::info;
use crate::utils::run_privileged_command;

pub async fn flash(image_ref: String, device_path: String) -> Result<()> {
    // Basic sanity check to avoid obvious command injection
    if image_ref.contains('"') || image_ref.contains('$') || device_path.contains('"') {
        anyhow::bail!("Invalid characters in input");
    }

    info!("Starting privileged flash operation for {} to {}", image_ref, device_path);

    // separator logic for partitions
    let sep = if device_path.chars().last().unwrap_or(' ').is_numeric() { "p" } else { "" };

    // Construct the bash script
    let script = format!(r#"
set -e
DEVICE="{device}"
IMAGE="{image}"
SEP="{sep}"
MOUNT_POINT="/run/bootc-media-creator/mnt"

# Ensure we have a random UUID for the disk label if possible, or just let sfdisk generate one if we omit label-id (default behavior is random).
# bootc explicitly sets it, so let's allow default random.

echo "Wiping filesystem signatures on $DEVICE..."
wipefs -a "$DEVICE"

echo "Partitioning $DEVICE..."
sfdisk "$DEVICE" <<EOF
label: gpt
# BIOS boot
size=1MiB, bootable, type=21686148-6449-6E6F-744E-656564454649, name="BIOS-BOOT"
# EFI System
size=512MiB, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, name="EFI-SYSTEM"
# Root
type=4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709, name="root"
EOF

udevadm settle

P_EFI="${{DEVICE}}${{SEP}}2"
P_ROOT="${{DEVICE}}${{SEP}}3"

echo "Formatting EFI ($P_EFI)..."
mkfs.fat -F 32 -n EFI-SYSTEM "$P_EFI"

echo "Formatting Root ($P_ROOT)..."
mkfs.xfs -f -L root "$P_ROOT"

echo "Mounting..."
mkdir -p "$MOUNT_POINT"
mount "$P_ROOT" "$MOUNT_POINT"

mkdir -p "$MOUNT_POINT/boot/efi"
mount "$P_EFI" "$MOUNT_POINT/boot/efi"

echo "Installing bootc image..."
# Check if bootc command exists
if ! command -v bootc &> /dev/null; then
    echo "bootc command not found"
    exit 1
fi

bootc install to-filesystem --source "$IMAGE" --skip-fetch-check "$MOUNT_POINT"

echo "Unmounting..."
umount -R "$MOUNT_POINT"
echo "Done!"
"#, 
    device = device_path,
    image = image_ref,
    sep = sep
    );

    // Run via pkexec bash -c "script"
    run_privileged_command("bash", &["-c", &script])?;

    Ok(())
}
