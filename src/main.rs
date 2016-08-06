extern crate chrono;
extern crate clap;
extern crate hyper;
extern crate iron;
extern crate params;
extern crate persistent;
extern crate router;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Debug};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::Local;
use clap::{Arg, App};

use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};

use iron::AfterMiddleware;
use iron::modifiers::Header;
use iron::typemap::Key;
use iron::prelude::*;
use iron::status;

use params::{Params, Value};
use persistent::Read;
use router::Router;
use rustc_serialize::json::{self, ToJson, Json};

const HELP_MSG: &'static str = "
<html>
    <head>
        <title>JS error logger server</title>
    </head>
    <body>
        <h1>JS error logger server.</h1>
        <p>
            For details, please visit <a href=\"https://github.com/lunemec/rust-http-logger\">https://github.com/lunemec/rust-http-logger</a>.
        </p>
    </body>
</html>";
const LOG_LEVELS: &'static [&'static str] = &["debug", "info", "warning", "error"];

#[derive(Copy, Clone)]
pub struct Log;
impl Key for Log { type Value = String; }

struct ErrorRecover;

#[derive(Debug)]
struct StringError(String);

#[derive(RustcEncodable)]
struct JsonResponse {
    success: Json,
    errors: Json
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

impl AfterMiddleware for ErrorRecover {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        println!("{} caught in ErrorRecover AfterMiddleware. {:?}", err.error, err);
        match err.response.status {
            Some(status::BadRequest) => Ok(err.response.set(status::Ok)),
            _ => Err(err)
        }
    }
}

fn open_log(log_path: &String) -> File {
    let path = Path::new(log_path);
    let display = path.display();

    // Open/create LOG_FILE for writing. Panic if unable to open/create.
    let file = match OpenOptions::new()
            .read(false)
            .append(true)
            .write(true)
            .create(true)
            .open(path) {
        Err(reason) => panic!("Unable to open log file {}. Reason: {}", display,
                                                                        reason.description()),
        Ok(file) => file,
    };
    file
}

fn help(_: &mut Request) -> IronResult<Response> {
    let content_type = Header(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
    Ok(Response::with((status::Ok, HELP_MSG, content_type)))
}

fn log_data(req: &mut Request) -> IronResult<Response> {
    // This capacity is used to create HashMaps without the need to re-allocate.
    let capacity = LOG_LEVELS.iter().count();
    // A HashMap containing Log_level -> bytes written for HTTP response.
    let mut success = HashMap::with_capacity(capacity);
    // A HashMap containing Log_level -> error reasons for HTTP response.
    let mut errors = HashMap::with_capacity(capacity);

    let arc = req.get::<Read<Log>>().unwrap();
    let log_path = arc.as_ref();

    let log_file = open_log(log_path);
    let mut log_writer = BufWriter::new(&log_file);

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
                // Try to write into the LOG_FILE. When successful, return HTTP 200 Ok with the number
                // of bytes writtern. Otherwise, we return HTTP 500 Internal Server Error with the
                // reason.
                match log_writer.write(&log_line.into_bytes()) {
                    Ok(bytes) => {success.insert(log_level, format!("{}", bytes));},
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

fn main() {
    let matches = App::new("HTTP JS error logger server.")
                          .version("1.0")
                          .author("Lukas Nemec <lu.nemec@gmail.com>")
                          .about("Listens on given address:port and logs POST requests to URL /log/ into the logfile.")
                          .arg(Arg::with_name("ADDRESS")
                               .required(true)
                               .help("Sets IP:PORT where to listen for connections (localhost:3000)"))
                          .arg(Arg::with_name("LOG")
                               .help("Sets the path to log file.")
                               .required(true))
                          .arg(Arg::with_name("serve_help")
                               .help("Will serve API help for this server, basically HTTP usage.")
                               .long("api_help")
                               .takes_value(false)
                          )
                          .get_matches();

    let address = matches.value_of("ADDRESS").unwrap();
    let log_path = matches.value_of("LOG").unwrap();
    let serve_help = matches.is_present("serve_help");

    let mut router = Router::new();

    router.post("/log/", log_data);

    if serve_help {
        router.any("/", help);
    } else {
        router.any("/", |_: &mut Request| {Ok(Response::with(status::Forbidden))});
    }

    let mut chain = Chain::new(router);
    chain.link(Read::<Log>::both(log_path.to_string()));
    chain.link_after(ErrorRecover);

    println!("Listening on {}", address);
    Iron::new(chain).http(address).unwrap();
}