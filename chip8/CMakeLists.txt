project(chip8)
add_executable(native_emus src/main.c)
target_link_libraries(native_emus ${SDL2_LIBRARY})
install(TARGETS native_emus RUNTIME DESTINATION ${BIN_DIR})
