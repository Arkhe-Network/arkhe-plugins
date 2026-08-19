#[derive(Clone)]
pub struct ThreadIndex {}
impl ThreadIndex {
    pub async fn get_usage_metrics(&self, _id: &str) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
}
