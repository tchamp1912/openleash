fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["proto/openleash/v1/openleash.proto"],
            &["proto"],
        )?;
    Ok(())
}