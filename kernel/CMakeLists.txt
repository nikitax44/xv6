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

# Define the kernel target
add_library(KernelLib STATIC ${OBJS})
