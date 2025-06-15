use prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/oanda_stream.proto");

    Config::new()
        .compile_protos(
            &["proto/oanda_stream.proto"],
            &[
                "proto/",
                "protobuf_includes/",
            ],
        )?;

    Ok(())
}
