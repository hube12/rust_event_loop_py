cmake_minimum_required(VERSION 3.20)
project(python_rust_example C)

set(CMAKE_C_STANDARD 11)

include_directories(../target)

add_executable(python_rust_example example.c)
target_include_directories(python_rust_example "../target")
target_link_libraries(python_rust_example "../target/release/libexample_rs.so")
