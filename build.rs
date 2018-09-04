///////// stepancheg/grpc-rust 版 /////////
extern crate protoc_rust_grpc;

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src",
        input: &["proto/hello.proto"],
        rust_protobuf: true,
        ..Default::default()
    }).expect("protoc-rust-grpc");
}


// ///////// pingcap/grpc-rs 版 /////////
// extern crate protoc_grpcio;

// fn main() {
//     let proto_root = "proto";
//     let output = "src/grpc_rs";
//     println!("cargo:rerun-if-changed={}", proto_root);
//     protoc_grpcio::compile_grpc_protos(
//         &["proto/hello.proto"],
//         &[proto_root],
//         &output
//     ).expect("Failed to compile gRPC definitions!");
// }
