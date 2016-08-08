use std::collections::HashMap;
use std::error::Error;

use chrono::Local;

use iron::prelude::*;
use iron::status;

use params::{Params, Value};
use persistent::Write;
use rustc_serialize::json::{self, ToJson};

use global::{LOG_LEVELS, JsonResponse, DataSender};


pub fn log_data(req: &mut Request) -> IronResult<Response> {
    // This capacity is used to create HashMaps without the need to re-allocate.
    let capacity = LOG_LEVELS.iter().count();
    // A HashMap containing Log_level -> bytes written for HTTP response.
    let mut success = HashMap::with_capacity(capacity);
    // A HashMap containing Log_level -> error reasons for HTTP response.
    let mut errors = HashMap::with_capacity(capacity);

    let mutex = req.get::<Write<DataSender>>().unwrap();
    let sender = mutex.lock().unwrap();

    let params = req.get_ref::<Params>();
    let params_map = match params {
        Ok(params) => params,
        Err(error) => return Ok(Response::with((status::BadRequest, error.description())))
    };

    // Iterate over accepted log levels and log them.
    for log_level in LOG_LEVELS {
        match params_map.get(*log_level) {
            Some(&Value::String(ref value)) => {
                let log_level = log_level.to_string();
                let log_level_upper = log_level.to_uppercase();
                let log_line = format!("[{datetime}] [{level}] {record}\n", datetime=Local::now().to_string(), level=log_level_upper, record=value);
                let bytes = log_line.len();
                // Try to write into the LOG_FILE. When successful, return HTTP 200 Ok with the number
                // of bytes writtern. Otherwise, we return HTTP 500 Internal Server Error with the
                // reason.
                match sender.send(log_line) {
                    Ok(_) => {success.insert(log_level, format!("{}", bytes));},
                    Err(reason) => {errors.insert(log_level, format!("{}", reason.description()));}
                }
            }
            _ => {}
        }
    }

    create_response(success, errors)
}

fn create_response(success: HashMap<String, String>, errors: HashMap<String, String>) -> IronResult<Response> {
    if success.is_empty() && errors.is_empty() {
        return Ok(Response::with((status::BadRequest, "Missing one of ['debug', 'info', 'warning', 'error'] in POST data.")));
    }

    let response = JsonResponse {
        success: success.to_json(),
        errors: errors.to_json(),
    };
    let data: String = json::encode(&response).unwrap();
    // If errors are not empty, we return different status code: 206 PartialContent
    if !errors.is_empty() {
        return Ok(Response::with((status::PartialContent, data)))
    }
    // Otherwise return 200 Ok.
    Ok(Response::with((status::Ok, data)))
}