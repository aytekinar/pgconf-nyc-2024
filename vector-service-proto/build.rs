fn main() -> std::io::Result<()> {
    // compile the proto file
    tonic_build::configure().compile_protos(&["proto/vector.proto"], &["proto"])
}
