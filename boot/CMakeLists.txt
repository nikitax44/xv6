cmake_minimum_required(VERSION 3.10)
project(KernelProject C ASM)

# Set the C standard
set(CMAKE_C_STANDARD 11)

set(C_WARNINGS "-Wall -Werror -Wpedantic -Wextra -Wold-style-definition -Wno-builtin-declaration-mismatch")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${C_WARNINGS}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -ggdb -gdwarf-2")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcmodel=medany -march=rv64g -static")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostdlib -nodefaultlibs -nostdinc -fno-builtin")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -MD")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ffreestanding")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}")


if (OPENSBI_ENABLED)
  set(BOOT sbi_entry.S sbi_start.c)
  set(KERNEL_ENTRY_START "0x80200000")
else ()
  set(BOOT m_entry.S m_start.c)
  set(KERNEL_ENTRY_START "0x80000000")
endif ()

add_custom_command(
  OUTPUT _kernel.ld
  COMMAND sed "s/@KERNEL_ENTRY_START@/${KERNEL_ENTRY_START}/g" < ${CMAKE_CURRENT_SOURCE_DIR}/kernel.ld.tmpl > _kernel.ld
  DEPENDS kernel.ld.tmpl
)
add_custom_target(KernelDeps ALL DEPENDS _kernel.ld)

if (DEFINED RUST_XV6)
  add_library(RustXv6 STATIC IMPORTED)
  set_target_properties(RustXv6 PROPERTIES IMPORTED_LOCATION ${RUST_XV6})
endif ()
if (DEFINED KERNEL)
  add_library(KernelLib STATIC IMPORTED)
  set_target_properties(KernelLib PROPERTIES IMPORTED_LOCATION ${KERNEL})
endif ()


add_executable(kernel ${BOOT})
target_link_libraries(kernel PRIVATE KernelLib RustXv6 KernelLib)
target_link_options(kernel PRIVATE "-T${CMAKE_CURRENT_BINARY_DIR}/_kernel.ld")
add_dependencies(kernel KernelDeps)

include_directories(${CMAKE_CURRENT_BINARY_DIR})

