cmake_minimum_required(VERSION 3.10)
project(KernelProject C ASM)

# Set the C standard
set(CMAKE_C_STANDARD 11)

# Define the source files (object files)
set(OBJS
    main.c

    fs/fs.c
    file/pipe.c
    file/file.c

    uapi/syscall.c
    uapi/sysfile.c

    hardware/uart.c
    hardware/disk/virtio_disk.c
    hardware/plic.c

    scheduler/list.c
    scheduler/proc.c

    util/spinlock.c
    util/sleeplock.c
    util/string.c
    util/util.c

    hardware/console.c
    vm.c
    context_switch/trap.c
    hardware/disk/bio.c
    fs/log.c
    exec.c

    context_switch/swtch.S
    context_switch/trampoline.S
    context_switch/kernelvec.S
    user_initcode.S
)

# Set the compiler flags
set(C_WARNINGS "-Wall -Werror -Wpedantic -Wextra -Wold-style-definition -Wno-builtin-declaration-mismatch")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${C_WARNINGS}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -ggdb -gdwarf-2")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcmodel=medany -march=rv64g -static")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostdlib -nodefaultlibs -nostdinc -fno-builtin")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -MD")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ffreestanding")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if (OPENSBI_ENABLED)
  set(BOOT boot/sbi_entry.S boot/sbi_start.c)
  set(KERNEL_ENTRY_START "0x80200000")
else ()
  set(BOOT boot/m_entry.S boot/m_start.c)
  set(KERNEL_ENTRY_START "0x80000000")
endif ()

add_custom_target(KernelDeps ALL DEPENDS _kernel.ld)

# Define the kernel target
add_library(KernelLib ${OBJS})
target_link_libraries(KernelLib)

if (DEFINED RUST_XV6)
  add_custom_target(GenerateLibRustXv6)

  add_library(RustXv6 STATIC IMPORTED)
  set_target_properties(RustXv6 PROPERTIES IMPORTED_LOCATION ${RUST_XV6})
endif ()

add_executable(kernel ${BOOT})
target_link_libraries(kernel PRIVATE KernelLib RustXv6 KernelLib)
target_link_options(kernel PRIVATE "-T${CMAKE_CURRENT_BINARY_DIR}/_kernel.ld")
add_dependencies(kernel KernelDeps GenerateLibRustXv6)

include_directories(${CMAKE_CURRENT_BINARY_DIR})

add_custom_command(
  OUTPUT _kernel.ld
  COMMAND sed "s/@KERNEL_ENTRY_START@/${KERNEL_ENTRY_START}/g" < ${CMAKE_CURRENT_SOURCE_DIR}/kernel.ld.tmpl > _kernel.ld
  DEPENDS kernel.ld.tmpl
)

