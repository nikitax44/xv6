cmake_minimum_required(VERSION 3.13)
project(MyProject C ASM)

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
        _shutdown
)

list(APPEND PORTS
        _test
        _dump
)

# Set the compiler flags
set(C_WARNINGS "-Wall -Werror -Wpedantic -Wextra -Wold-style-definition")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${C_WARNINGS}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -ggdb -gdwarf-2")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcmodel=medany -march=rv64g -static")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I ${NEWLIB}/include --specs=${NEWLIB}/lib/nano.specs")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I ${CMAKE_CURRENT_SOURCE_DIR}/../")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -MD")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-builtin-log -fno-builtin-printf -fno-builtin-fprintf -fno-builtin-vprintf -fno-builtin-putc")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}")

set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -z max-page-size=4096 -L ${NEWLIB}/lib -nostdlib")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Custom command to generate usys.o from usys.pl
add_custom_command(
        OUTPUT _usys.S
        COMMAND perl ${CMAKE_CURRENT_SOURCE_DIR}/usys.pl > _usys.S
        DEPENDS usys.pl
        COMMENT "Generating _usys.S from usys.pl"
)

add_library(u STATIC
        ulib.c
        _usys.S
        printf.c
)

# Add the executable targets for user programs
foreach (PROG ${UPROGS})
    string(REGEX REPLACE "^_" "" BASENAME "${PROG}")
    add_executable(${PROG} start.c ${BASENAME}.c)
    target_link_libraries(${PROG} u -lc)
    target_link_options(${PROG} PRIVATE "-T${CMAKE_CURRENT_SOURCE_DIR}/user.ld")
endforeach ()

# Add the executable targets for user programs
foreach (PROG ${PORTS})
    string(REGEX REPLACE "^_" "" BASENAME "${PROG}")
    add_executable(${PROG} ports/fixes.c ports/${BASENAME}.c)
    target_link_libraries(${PROG} u -lc)
    target_link_options(${PROG} PRIVATE "-T${CMAKE_CURRENT_SOURCE_DIR}/ports/app.ld")
endforeach ()

add_custom_command(
        OUTPUT fs.tar
        COMMAND tar -cf fs.tar ${UPROGS} ${PORTS}
        COMMENT "Packing the programs to .tar"
        DEPENDS ${UPROGS} ${PORTS}
)

add_custom_target(Programs ALL
        DEPENDS fs.tar
)
