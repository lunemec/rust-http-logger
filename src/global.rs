use std::sync::mpsc::Sender;

use iron::typemap::Key;

use rustc_serialize::json::Json;

pub const LOG_LEVELS: &'static [&'static str] = &["debug", "info", "warning", "error"];

#[derive(RustcEncodable)]
pub struct JsonResponse {
    pub success: Json,
    pub errors: Json
}

#[derive(Copy, Clone)]
pub struct DataSender;
impl Key for DataSender { type Value = Sender<String>; }