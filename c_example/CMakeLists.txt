cmake_minimum_required(VERSION 3.15)
project(c_example LANGUAGES C)

add_subdirectory(corrosion)

corrosion_import_crate(MANIFEST_PATH ../Cargo.toml)

add_executable(example example.c)
include_directories(${CMAKE_SOURCE_DIR}/..)
message(STATUS "build dir is" ${CMAKE_BINARY_DIR})
target_link_directories(example PUBLIC ${CMAKE_BINARY_DIR})
target_link_libraries(example PUBLIC demes_forward_ffi)
