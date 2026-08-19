pub struct TraceManager {}
impl TraceManager {
    pub async fn add_artifact(
        &self,
        _trace_id: &str,
        _name: &str,
        _data: Vec<u8>,
        _mime: &str,
        _desc: &str,
    ) -> Result<(), String> {
        Ok(())
    }
    pub async fn start_trace(&self, _resource_id: &str) -> Result<String, String> {
        Ok("trace".to_string())
    }
}
pub enum SpanStatus {
    Ok,
    Error,
}
pub enum TraceLevel {
    Info,
    Warn,
    Error,
}
