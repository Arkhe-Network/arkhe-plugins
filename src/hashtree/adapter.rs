#[derive(Clone)]
pub struct HashTreeStorage {}
impl HashTreeStorage {
    pub async fn get_by_path(&self, _path: &str) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
    pub async fn put(&self, _data: &[u8]) -> Result<String, String> {
        Ok("hash".to_string())
    }
}
