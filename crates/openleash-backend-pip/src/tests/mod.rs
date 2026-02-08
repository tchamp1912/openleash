#[cfg(test)]
mod tests {
    use crate::PipBackend;
    use openleash_backend::PackageBackend;

    #[tokio::test]
    async fn test_pip_executable_dir() {
        let backend = PipBackend;
        let dir = "/tmp/venv";
        let exe_dir = backend.executable_directory(dir);
        assert!(exe_dir.to_string_lossy().contains("bin") || exe_dir.to_string_lossy().contains("Scripts"));
    }
}
