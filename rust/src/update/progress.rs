//! Fixed CLI management progress on stderr; never JSON-RPC stdout or saved settings.
use serde_json::json;
pub(crate) fn stage(stage: &str, message: &str) {
    eprintln!(
        "{}",
        json!({"event":"progress","stage":stage,"message":message})
    );
}
pub(crate) fn download(received: u64, total: Option<u64>) {
    eprintln!(
        "{}",
        json!({"event":"progress","stage":"download","received_bytes":received,"total_bytes":total})
    );
}
