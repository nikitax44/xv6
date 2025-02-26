cmake_minimum_required(VERSION 3.10)
project(Xv6 C ASM)

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

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I ${CMAKE_SOURCE_DIR}/include")

# Add the mkfs target
add_custom_command(
  OUTPUT mkfs.elf
  COMMAND gcc -o mkfs.elf ${CMAKE_CURRENT_SOURCE_DIR}/mkfs/mkfs.c -I ${CMAKE_CURRENT_SOURCE_DIR}/include
  DEPENDS mkfs/mkfs.c include/kernel/file/fs.h include/kernel/file/stat.h include/kernel/param.h include/kernel/types.h
  COMMENT "Building mkfs with host compiler"
)

set(CARGO_BASE_OPTS -C ${CMAKE_SOURCE_DIR}/rust-xv6 -Z unstable-options)
set(CARGO_EXTRA_OPTS --target-dir ${CMAKE_BINARY_DIR}/target --release --frozen --lib)
set(CARGO_ARGUMENTS ${CARGO_EXTRA_OPTS} --artifact-dir ${CMAKE_BINARY_DIR})
# set(CARGO_ARGUMENTS ${CARGO_ARGUMENTS} --features talc)
add_custom_command(
  OUTPUT ${CMAKE_BINARY_DIR}/librust_xv6.a
  COMMAND cargo ${CARGO_BASE_OPTS} clippy ${CARGO_EXTRA_OPTS} || true
  COMMAND cargo ${CARGO_BASE_OPTS} build ${CARGO_ARGUMENTS}
  DEPENDS ${CMAKE_SOURCE_DIR}/rust-xv6/src/** ${CMAKE_SOURCE_DIR}/rust-xv6/Cargo.* Bindgen
  BYPRODUCTS ${CMAKE_BINARY_DIR}/target
  COMMENT "Building librust_xv6.a"
)

set(BINDGEN_OPTS
    --use-core
    --wrap-static-fns
    --wrap-unsafe-ops
    --emit-diagnostics
    --experimental
    --no-derive-copy
    --no-debug
    trapframe
)
add_custom_command(
  OUTPUT ${CMAKE_SOURCE_DIR}/rust-xv6/xv6_binds.rs
  COMMAND fd -e h --base-directory "${CMAKE_SOURCE_DIR}/include" --strip-cwd-prefix -x echo "#include \"{}\"" >
          ${CMAKE_BINARY_DIR}/_everything.h
  VERBATIM
  COMMAND bindgen ${CMAKE_BINARY_DIR}/_everything.h -o ${CMAKE_SOURCE_DIR}/rust-xv6/xv6_binds.rs ${BINDGEN_OPTS} -- -D
          __ASSEMBLER__ -I "${CMAKE_SOURCE_DIR}/include" -nostdinc
  BYPRODUCTS ${CMAKE_BINARY_DIR}/_everything.h
  DEPENDS ${CMAKE_SOURCE_DIR}/include/kernel/* ${CMAKE_SOURCE_DIR}/include/kernel/*/*
)

add_custom_target(Bindgen DEPENDS rust-xv6/xv6_binds.rs)

add_custom_target(GenerateLibRustXv6 DEPENDS librust_xv6.a)
add_library(RustXv6 STATIC IMPORTED)
set_target_properties(RustXv6 PROPERTIES IMPORTED_LOCATION ${CMAKE_CURRENT_BINARY_DIR}/librust_xv6.a)

add_subdirectory(kernel)
add_subdirectory(user)
add_subdirectory(boot)

# Add the filesystem image target
add_custom_command(
  OUTPUT fs.img
  COMMAND rm -rf temp_files && mkdir temp_files
  COMMAND tar -C temp_files -xf ${CMAKE_BINARY_DIR}/user/fs.tar
  COMMAND ${CMAKE_BINARY_DIR}/mkfs.elf fs.img temp_files/* "${CMAKE_SOURCE_DIR}/README"
  BYPRODUCTS temp_files
  DEPENDS ${CMAKE_BINARY_DIR}/mkfs.elf Programs ${CMAKE_SOURCE_DIR}/README
)

add_custom_target(mkfs DEPENDS mkfs.elf)

add_custom_target(image DEPENDS fs.img)

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

add_custom_target(script DEPENDS qemu-script)

# Add the QEMU target
add_custom_target(
  qemu
  DEPENDS kernel fs.img qemu-script
  COMMAND CPUS=${NPROC} KERNEL="${CMAKE_BINARY_DIR}/boot/kernel" FS="${CMAKE_CURRENT_BINARY_DIR}/fs.img" ./qemu-script
)

# Generate GDB initialization file
add_custom_command(
  OUTPUT ${CMAKE_SOURCE_DIR}/.gdbinit
  COMMAND sed -e "s/:1234/:${GDBPORT}/" -e "s#build/kernel#${CMAKE_BINARY_DIR}/boot/kernel#" <
          ${CMAKE_CURRENT_SOURCE_DIR}/.gdbinit.tmpl-riscv > ${CMAKE_SOURCE_DIR}/.gdbinit
  DEPENDS .gdbinit.tmpl-riscv
)

# Add the GDB target
add_custom_target(
  qemu-gdb
  DEPENDS kernel ${CMAKE_SOURCE_DIR}/.gdbinit fs.img qemu-script
  COMMAND ${CMAKE_COMMAND} -E echo "*** Now run 'gdb' in another window."
  COMMAND CPUS=1 KERNEL="${CMAKE_CURRENT_BINARY_DIR}/boot/kernel" FS="${CMAKE_CURRENT_BINARY_DIR}/fs.img" ./qemu-script
          -S -gdb "tcp::${GDBPORT}"
)
