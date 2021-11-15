#include <memory>
#include <iostream>
#include <string>
#include <thread>

#include <grpcpp/grpcpp.h>
#include <grpc/support/log.h>

#ifdef BAZEL_BUILD
#include "proto/hello.grpc.pb.h"
#else

#include "hello.grpc.pb.h"

#endif

using grpc::Server;
using grpc::ServerAsyncResponseWriter;
using grpc::ServerBuilder;
using grpc::ServerCompletionQueue;
using grpc::ServerContext;
using grpc::Status;
using hello::Hello;
using hello::HelloRequest;
using hello::HelloResponse;

class ServerImpl final {
public:
    ~ServerImpl()
    {
        server_->Shutdown();

        for (auto &cq: cq_)
            cq->Shutdown();
    }

    void Run()
    {
        std::string server_address("0.0.0.0:5000");

        ServerBuilder builder;

        builder.AddListeningPort(server_address, grpc::InsecureServerCredentials());
        builder.RegisterService(&service_);

        auto parallelism = std::atoi(std::getenv("GOMAXPROCS"));
        for (int i = 0; i < parallelism; i++) {
            cq_.emplace_back(builder.AddCompletionQueue());
        }

        server_ = builder.BuildAndStart();
        std::cout << "Server listening on " << server_address << std::endl;

        for (unsigned int i = 0; i < parallelism; i++) {
            server_threads_.emplace_back(std::thread(
                [this, i] {
                    cpu_set_t cpuset;
                    CPU_ZERO(&cpuset);
                    CPU_SET(i, &cpuset);
                    pthread_setaffinity_np(pthread_self(), sizeof(cpu_set_t), &cpuset);
                    this->HandleRpcs(cq_[i].get());
                }));
        }

        std::this_thread::sleep_until(std::chrono::time_point<std::chrono::system_clock>::max());
    }

private:
    class CallData {
    public:
        CallData(Hello::AsyncService *service, ServerCompletionQueue *cq)
            : service_(service), cq_(cq), responder_(&ctx_)
        {
            service_->RequestSayHello(&ctx_, &request_, &responder_, cq_, cq_,
                                      this);
        }

        void Proceed()
        {
            if (status_ == PROCESS) {
                reply_.set_greetings(request_.name());

                status_ = FINISH;
                responder_.Finish(reply_, Status::OK, this);
            } else {
                GPR_ASSERT(status_ == FINISH);

                service_->RequestSayHello(&ctx_, &request_, &responder_, cq_, cq_,
                                          this);
                ctx_.~ServerContext();
                responder_ = ServerAsyncResponseWriter<HelloResponse>(new(&ctx_)ServerContext());

                status_ = PROCESS;
            }
        }

    private:
        Hello::AsyncService *service_;
        ServerCompletionQueue *cq_;
        ServerContext ctx_;
        HelloRequest request_;
        HelloResponse reply_;
        ServerAsyncResponseWriter<HelloResponse> responder_;

        enum CallStatus {
            PROCESS,
            FINISH
        };
        CallStatus status_ = PROCESS;
    };

    void HandleRpcs(ServerCompletionQueue *cq)
    {
        std::vector<std::unique_ptr<CallData>> ctx{};
        ctx.reserve(1000);
        for (int i = 0; i < 1000; ++i) {
            ctx.push_back(std::make_unique<CallData>(&service_, cq));
        }
        void *tag;
        bool ok;
        while (true) {

            GPR_ASSERT(cq->Next(&tag, &ok));
            static_cast<CallData *>(tag)->Proceed();
        }
    }

    std::vector<std::unique_ptr<ServerCompletionQueue>> cq_;
    Hello::AsyncService service_;
    std::unique_ptr<Server> server_;
    std::vector<std::thread> server_threads_;
};

int main(int argc, char **argv)
{
    ServerImpl server;
    server.Run();

    return 0;
}
