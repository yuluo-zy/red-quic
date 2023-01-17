pub fn digest(data: &[u8]) -> [u8;32] {
    use sha2::{Digest, Sha256};
    let d = Sha256::new().chain_update(data).finalize();
    d.into()
}
