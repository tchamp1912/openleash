fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["proto/leash/v1/leash.proto"],
            &["proto"],
        )?;
    Ok(())
}