cmake_minimum_required(VERSION 3.10)
project(Xv6 C ASM)

set(CMAKE_C_STANDARD 17)

if(NOT DEFINED GDBPORT)
  set(GDBPORT $ENV{GDBPORT})
  if(NOT DEFINED GDBPORT)
    set(GDBPORT 1235)
  endif()
endif()
if(NOT DEFINED NPROC)
  set(NPROC $ENV{NPROC})
  if(NOT DEFINED NPROC)
    set(NPROC 8)
  endif()
endif()
if(NOT DEFINED OPENSBI_ENABLED)
  set(OPENSBI_ENABLED $ENV{OPENSBI_ENABLED})
  if(NOT DEFINED OPENSBI_ENABLED)
    set(OPENSBI_ENABLED true)
  endif()
endif()

if(NOT DEFINED NEWLIB)
  set(NEWLIB $ENV{NEWLIB})
  if(NOT DEFINED NEWLIB)
    message(FATAL_ERROR "NEWLIB must be set and exported by shell")
  endif()
endif()

set(CARGO_ARGUMENTS --release --frozen --lib --target-dir ${CMAKE_CURRENT_BINARY_DIR}/target)
add_custom_command(
  OUTPUT librust_xv6.a
  COMMAND cd ./rust-xv6 && cargo build ${CARGO_ARGUMENTS}
  COMMAND cp ./target/riscv64gc-unknown-none-elf/release/librust_xv6.a ./
  DEPENDS ./rust-xv6/src/** ./rust-xv6/Cargo.*
  COMMENT "Building librust_xv6.a"
)

add_custom_target(GenerateLibRustXv6 ALL DEPENDS librust_xv6.a)

add_library(RustXv6 STATIC IMPORTED)
set_target_properties(RustXv6 PROPERTIES IMPORTED_LOCATION ${CMAKE_CURRENT_BINARY_DIR}/librust_xv6.a)

add_subdirectory(kernel)
add_subdirectory(user)

# Add the filesystem image target
add_custom_command(
  OUTPUT fs.img
  COMMAND virt-make-fs ${CMAKE_CURRENT_BINARY_DIR}/user/fs.tar fs.img
  DEPENDS Programs
)

add_custom_target(image ALL DEPENDS fs.img)

# Enable bios for qemu
if(OPENSBI_ENABLED)
  set(QEMU_FLAGS "")
  message("OpenSBI is enabled")
else()
  set(QEMU_FLAGS "-bios none")
  message("OpenSBI is disabled")
endif()

add_custom_command(
  OUTPUT qemu-script
  COMMAND sed "s/@QEMU_FLAGS@/${QEMU_FLAGS}/g" < "${CMAKE_CURRENT_SOURCE_DIR}/qemu-script.tmpl" > qemu-script
  COMMAND chmod a+x qemu-script
  DEPENDS qemu-script.tmpl
)

add_custom_target(script ALL DEPENDS qemu-script)

# Add the QEMU target
add_custom_target(
  qemu
  DEPENDS kernel fs.img qemu-script
  COMMAND CPUS=${NPROC} KERNEL="${CMAKE_CURRENT_BINARY_DIR}/kernel/kernel" FS="${CMAKE_CURRENT_BINARY_DIR}/fs.img"
          ./qemu-script
)

# Generate GDB initialization file
add_custom_command(
  OUTPUT .gdbinit
  COMMAND sed "s/:1234/:${GDBPORT}/" < ${CMAKE_CURRENT_SOURCE_DIR}/.gdbinit.tmpl-riscv > .gdbinit
  DEPENDS .gdbinit.tmpl-riscv
)

# Add the GDB target
add_custom_target(
  qemu-gdb
  DEPENDS kernel .gdbinit fs.img qemu-script
  COMMAND ${CMAKE_COMMAND} -E echo "*** Now run 'gdb' in another window."
  COMMAND CPUS=${NPROC} KERNEL="${CMAKE_CURRENT_BINARY_DIR}/kernel/kernel" FS="${CMAKE_CURRENT_BINARY_DIR}/fs.img"
          ./qemu-script -S -gdb "tcp::${GDBPORT}"
)
