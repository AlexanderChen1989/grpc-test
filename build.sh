#!/bin/bash
set -e


build() {
    pushd proto >/dev/null
    protoc --cpp_out=. hello.proto
    protoc --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_cpp_plugin` hello.proto
    popd >/dev/null

    g++ -march=native -O3 hello_server.cc proto/hello.*.cc -I proto/ -o hello_server_cpp -pthread `pkg-config --libs grpc++` -lprotobuf -ggdb -lgrpc++_reflection
    CGO_ENABLED=0 go build -o hello_server_go main.go
    cargo +nightly build --release
}

test_lang() {
    local name=$1
    shift
    echo "Running test for ${name} implementation"
    "$@" &
    pid=$!
    sleep 1 # Wait the process ready
    ghz --insecure \
    -n 200000 \
    --cpus=2 \
    --concurrency=500 \
    --connections=10 \
    --proto ./proto/hello.proto \
    --call hello.Hello.SayHello \
    -d '{"name":"Joe"}' \
    0.0.0.0:5000
    kill -TERM $pid
    wait $pid || true
}



run_test() {
    export GOMAXPROCS=$1
    echo "Testing with $1 threads" 
    test_lang C++ ./hello_server_cpp
    test_lang Go ./hello_server_go
    if [ "$GOMAXPROCS" -eq 1 ]; then
        test_lang Rust ./target/release/s_t
    else
        test_lang Rust ./target/release/m_t
    fi
}

build
run_test 1
run_test 2
run_test 4
run_test 8
