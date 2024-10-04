cmake_minimum_required(VERSION 3.10)
project(Xv6 C ASM)

set(GDBPORT 1234)
set(NPROC 8)


if(NOT DEFINED NEWLIB)
    set(NEWLIB $ENV{NEWLIB})
    if(NOT DEFINED NEWLIB)
        message(FATAL_ERROR "NEWLIB must be set and exported by shell")
    endif()
endif()

# Set compiler flags
set(C_WARNINGS "-Wall -Werror -Wpedantic -Wextra")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${C_WARNINGS}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -ggdb -gdwarf-2")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcmodel=medany -march=rv64g -static")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I ${CMAKE_CURRENT_SOURCE_DIR} -I ${NEWLIB}/include --specs=${NEWLIB}/lib/nano.specs")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -MD")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Disable PIE if possible
execute_process(COMMAND ${CMAKE_C_COMPILER} -dumpspecs OUTPUT_VARIABLE DUMPSPECS)
if(DUMPSPECS MATCHES "[^f]no-pie")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-pie -no-pie")
endif()
if(DUMPSPECS MATCHES "[^f]nopie")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-pie -nopie")
endif()

set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -z max-page-size=4096 -L ${NEWLIB}/lib -nostdlib")

# Define the programs
list(APPEND UPROGS
    _cat
    _echo
    _forktest
    _grep
    _init
    _kill
    _ln
    _ls
    _mkdir
    _rm
    _sh
    _stressfs
    _usertests
    _grind
    _wc
    _zombie
)

list(APPEND PORTS
    _test
    _dump
)

separate_arguments(C_WARNINGS)
# Add the mkfs target
add_custom_command(
    OUTPUT mkfs.elf
    COMMAND gcc ${C_WARNINGS} -o mkfs.elf ${CMAKE_CURRENT_SOURCE_DIR}/mkfs/mkfs.c -I ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS mkfs/mkfs.c kernel/file/fs.h kernel/file/stat.h kernel/param.h kernel/types.h
    COMMENT "Building mkfs with host compiler"
)


add_subdirectory(kernel)
add_subdirectory(user)

set(UUPROGS ${UPROGS})
list(TRANSFORM UUPROGS PREPEND "user/")
set(PPORTS ${PORTS})
list(TRANSFORM PPORTS PREPEND "user/")

# Add the filesystem image target
add_custom_command(
    OUTPUT fs.img
    COMMAND ./mkfs.elf fs.img ${UUPROGS} ${PPORTS} "${CMAKE_CURRENT_SOURCE_DIR}/README"
    DEPENDS mkfs.elf ${UUPROGS} ${PPORTS} "README"
)

add_custom_target(image ALL
    DEPENDS fs.img
)

# Add the QEMU target
add_custom_target(qemu
    DEPENDS kernel fs.img
    COMMAND qemu-system-riscv64
        -machine virt -m 128M -smp ${NPROC} -nographic
        -global "virtio-mmio.force-legacy=false"
        -drive "file=fs.img,if=none,format=raw,id=x0"
        -device "virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0"
        -kernel ${CMAKE_CURRENT_BINARY_DIR}/kernel/kernel
)

# Generate GDB initialization file
add_custom_command(
    OUTPUT .gdbinit
    COMMAND sed "s/:1234/:${GDBPORT}/" < ${CMAKE_CURRENT_SOURCE_DIR}/.gdbinit.tmpl-riscv > .gdbinit
    DEPENDS .gdbinit.tmpl-riscv
)

# Add the GDB target
add_custom_target(qemu-gdb
    DEPENDS kernel .gdbinit fs.img
    COMMAND ${CMAKE_COMMAND} -E echo "*** Now run 'gdb' in another window."
    COMMAND qemu-system-riscv64
        -machine virt -m 128M -smp ${NPROC} -nographic
        -global "virtio-mmio.force-legacy=false"
        -drive "file=fs.img,if=none,format=raw,id=x0"
        -device "virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0"
        -kernel ${CMAKE_CURRENT_BINARY_DIR}/kernel/kernel
        -S -gdb tcp::${GDBPORT}
)
