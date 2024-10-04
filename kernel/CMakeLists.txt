cmake_minimum_required(VERSION 3.10)
project(KernelProject C ASM)

# Set the C standard
set(CMAKE_C_STANDARD 11)

# Define the source files (object files)
set(OBJS
    boot/entry.S
    boot/start.c
    boot/main.c
    boot/dtb.c

    file/fs.c
    file/file.c

    uapi/syscall.c
    uapi/sysproc.c
    uapi/sysfile.c

    hardware/uart.c
    hardware/virtio_disk.c
    hardware/plic.c

    util/spinlock.c
    util/sleeplock.c
    util/printf.c
    util/string.c

    console.c
    kalloc.c
    vm.c
    proc.c
    trap.c
    bio.c
    log.c
    pipe.c
    exec.c

    swtch.S
    trampoline.S
    kernelvec.S
)

# Set compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ffreestanding")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-builtin-log -fno-builtin-printf -fno-builtin-fprintf -fno-builtin-vprintf -fno-builtin-putc")

add_custom_command(
    OUTPUT _initcode.h
    COMMAND ${CMAKE_ASM_COMPILER} -nostdinc -c ${CMAKE_CURRENT_SOURCE_DIR}/initcode.S -o initcode.o
    COMMAND ${CMAKE_LINKER} -N -e start -Ttext 0 -o initcode.out initcode.o
    COMMAND ${CMAKE_OBJCOPY} -S -O binary initcode.out initcode
    COMMAND xxd -i -n initcode initcode > _initcode.h
    COMMAND rm initcode.o initcode initcode.out
    DEPENDS initcode.S
    COMMENT "Generating _initcode.h from initcode.S"
)

add_custom_target(RebuildOnDependency ALL DEPENDS kernel.ld _initcode.h)

# Define the kernel target
add_executable(kernel ${OBJS})
target_link_libraries(kernel -lc)
add_dependencies(kernel RebuildOnDependency)
target_link_options(kernel PRIVATE "-T${CMAKE_CURRENT_SOURCE_DIR}/kernel.ld")

include_directories(${CMAKE_CURRENT_BINARY_DIR})
