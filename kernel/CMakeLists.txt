cmake_minimum_required(VERSION 3.10)
project(KernelProject C ASM)

# Set the C standard
set(CMAKE_C_STANDARD 11)

# Define the source files (object files)
set(OBJS
        main.c

        file/fs.c
        file/pipe.c
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
        util/util.c

        console.c
        vm.c
        proc.c
        trap.c
        bio.c
        log.c
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

set(CARGO_ARGUMENTS --release --frozen --lib --target-dir ${CMAKE_CURRENT_BINARY_DIR}/target)

if (OPENSBI_ENABLED)
    set(BOOT boot/sbi_entry.S boot/sbi_start.c)
else ()
    set(BOOT boot/m_entry.S boot/m_start.c)
endif ()
if (RUST_KALLOC_ENABLE)
    list(APPEND CARGO_ARGUMENTS --features rust_kalloc)
else ()
    list(APPEND OBJS kalloc.c)
endif ()

add_custom_command(
        OUTPUT librust_xv6.a
        COMMAND cd ../rust-xv6 && cargo build ${CARGO_ARGUMENTS}
        COMMAND cp target/riscv64gc-unknown-none-elf/release/librust_xv6.a ./
        DEPENDS ${CMAKE_SOURCE_DIR}/rust-xv6/src/** ${CMAKE_SOURCE_DIR}/rust-xv6/Cargo.*
        COMMENT "Building rust part of xv6"
)

add_custom_target(Initcode ALL DEPENDS _initcode.h)

add_custom_target(KernelDeps ALL DEPENDS _kernel.ld librust_xv6.a)

# Define the kernel target
add_library(kernelLib ${OBJS})
target_link_libraries(kernelLib -lc)
add_dependencies(kernelLib Initcode)

add_executable(kernel ${BOOT})
target_link_libraries(kernel kernelLib ${CMAKE_CURRENT_BINARY_DIR}/librust_xv6.a)
target_link_options(kernel PRIVATE "-T${CMAKE_CURRENT_BINARY_DIR}/_kernel.ld")
add_dependencies(kernel KernelDeps)

include_directories(${CMAKE_CURRENT_BINARY_DIR})

if (OPENSBI_ENABLED)
    set(KERNEL_ENTRY_START "0x80200000")
    add_compile_definitions(-DSBI_ENABLE)
else ()
    set(KERNEL_ENTRY_START "0x80000000")
endif ()

add_custom_command(
        OUTPUT _kernel.ld
        COMMAND sed "s/@KERNEL_ENTRY_START@/${KERNEL_ENTRY_START}/g" < ${CMAKE_CURRENT_SOURCE_DIR}/kernel.ld.tmpl > _kernel.ld
        DEPENDS kernel.ld.tmpl
)

if (OPENSBI_ENABLED)
endif ()
