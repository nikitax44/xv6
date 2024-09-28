cmake_minimum_required(VERSION 3.13)
project(MyProject C ASM)

# Set the compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-builtin-log -fno-builtin-printf -fno-builtin-fprintf -fno-builtin-vprintf -fno-builtin-putc")

# Custom command to generate usys.o from usys.pl
add_custom_command(
    OUTPUT _usys.S
    COMMAND perl ${CMAKE_CURRENT_SOURCE_DIR}/usys.pl > _usys.S
    DEPENDS usys.pl
    COMMENT "Generating _usys.S from usys.pl"
)

add_library(libu STATIC
    ulib.c
    _usys.S
    printf.c
)

add_library(libf STATIC
    ports/fixes.c
)


# Add the executable targets for user programs
foreach(PROG ${UPROGS})
    string(REGEX REPLACE "^_" "" BASENAME "${PROG}")
    add_executable(${PROG} ${BASENAME}.c )
    target_link_libraries(${PROG} libu -lc)
    target_link_options(${PROG} PRIVATE "-T${CMAKE_CURRENT_SOURCE_DIR}/user.ld")
endforeach()

# Add the executable targets for user programs
foreach(PROG ${PORTS})
    string(REGEX REPLACE "^_" "" BASENAME "${PROG}")
    add_executable(${PROG} ports/${BASENAME}.c)
    target_link_libraries(${PROG} libf libu -lc)
    target_link_options(${PROG} PRIVATE "-T${CMAKE_CURRENT_SOURCE_DIR}/ports/app.ld")
endforeach()
