cmake_minimum_required(VERSION 3.0)
project(grpc-test)
include(FindPkgConfig)
pkg_check_modules (GRPC REQUIRED grpc++ protobuf)

add_executable(hello_server_cpp
        hello_server.cc
        proto/hello.grpc.pb.cc
        proto/hello.pb.cc
        )
target_link_options(hello_server_cpp PUBLIC ${GRPC_LDFLAGS} -lgrpc++_reflection)
target_include_directories(hello_server_cpp PUBLIC proto)