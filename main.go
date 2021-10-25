package main

import (
	"context"
	"log"
	"net"
	"os"
	"strconv"

	"golang.org/x/sys/unix"
	"google.golang.org/grpc"
	"hello.grpc/proto"
)

type Hello struct {
	proto.UnimplementedHelloServer
}

func (h *Hello) SayHello(
	ctx context.Context,
	in *proto.HelloRequest,
) (*proto.HelloResponse, error) {
	return &proto.HelloResponse{Greetings: in.Name}, nil
}

func main() {
	var newMask unix.CPUSet
	n, _ := strconv.Atoi(os.Getenv("GOMAXPROCS"))
	for i := 0; i < n; i++ {
		newMask.Set(i)
	}
	unix.SchedSetaffinity(0, &newMask);

	lis, err := net.Listen("tcp", ":5000")
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}
	s := grpc.NewServer()
	proto.RegisterHelloServer(s, &Hello{})
	log.Printf("server listening at %v", lis.Addr())
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
