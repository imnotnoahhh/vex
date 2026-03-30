use crate::config;
use crate::error::{Result, VexError};
use sha2::Digest;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn sha256_hex(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0u8; config::CHECKSUM_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(encode_hex(hasher.finalize().as_ref()))
}

pub(crate) fn verify_sha256(file_path: &Path, expected: &str) -> Result<()> {
    let actual = sha256_hex(file_path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(VexError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}
