# Hyper-V example: Limine boot, ISO image, QEMU target
# Limine binaries are fetched in the root CMakeLists.txt

set(HYPERV_KERNEL_REL "target/${TARGET}/release/embclox-hyperv-example")

add_custom_target(hyperv-build
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Hyper-V example (Limine BIOS+UEFI)"
)

# --- ISO image (for QEMU and local Hyper-V Gen1) ---

set(HYPERV_ISO "${CMAKE_BINARY_DIR}/hyperv.iso")

add_custom_command(
    OUTPUT ${HYPERV_ISO}
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/hyperv-iso/boot/limine
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/hyperv-iso/EFI/BOOT
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_SOURCE_DIR}/${HYPERV_KERNEL_REL}
        ${CMAKE_BINARY_DIR}/hyperv-iso/boot/kernel
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_CURRENT_SOURCE_DIR}/limine.conf
        ${CMAKE_BINARY_DIR}/hyperv-iso/boot/limine/limine.conf
    # UEFI boot files
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LIMINE_BOOTX64}
        ${CMAKE_BINARY_DIR}/hyperv-iso/EFI/BOOT/BOOTX64.EFI
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LIMINE_UEFI_CD}
        ${CMAKE_BINARY_DIR}/hyperv-iso/boot/limine/limine-uefi-cd.bin
    # BIOS boot files (needed for Hyper-V Gen1)
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LIMINE_BIOS_CD}
        ${CMAKE_BINARY_DIR}/hyperv-iso/boot/limine/limine-bios-cd.bin
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LIMINE_BIOS_SYS}
        ${CMAKE_BINARY_DIR}/hyperv-iso/boot/limine/limine-bios.sys
    COMMAND xorriso -as mkisofs
        -b boot/limine/limine-bios-cd.bin
        -no-emul-boot -boot-load-size 4 -boot-info-table
        --efi-boot boot/limine/limine-uefi-cd.bin
        -efi-boot-part --efi-boot-image --protective-msdos-label
        ${CMAKE_BINARY_DIR}/hyperv-iso -o ${HYPERV_ISO}
        2>/dev/null
    DEPENDS hyperv-build ${LIMINE_BOOTX64}
    COMMENT "Creating Hyper-V BIOS+UEFI ISO image"
)
add_custom_target(hyperv-image DEPENDS ${HYPERV_ISO})

# --- QEMU run target (VMBus won't work, but tests basic boot) ---

add_custom_target(qemu-hyperv
    COMMAND qemu-system-x86_64
        -machine q35
        -no-reboot
        -serial mon:stdio
        -display none
        -drive if=pflash,format=raw,readonly=on,file=${OVMF_CODE}
        -cdrom ${HYPERV_ISO}
        -m 256M
    DEPENDS hyperv-image
    COMMENT "Running Hyper-V example in QEMU (UEFI, no VMBus)"
)

# --- CTest: boot test on QEMU (verifies Limine boot, non-Hyper-V path) ---

add_test(NAME hyperv-boot
    COMMAND bash ${CMAKE_SOURCE_DIR}/scripts/qemu-test.sh
        ${HYPERV_ISO}
        --iso
        --log-match "HYPERV BOOT PASSED"
        --timeout 15
        --qemu-args "-m 256M"
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)
