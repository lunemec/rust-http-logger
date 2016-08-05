extern crate chrono;
extern crate clap;
extern crate iron;
extern crate router;
extern crate params;
extern crate persistent;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::Local;
use clap::{Arg, App};

use iron::typemap::Key;
use iron::prelude::*;
use iron::status;

use router::Router;
use params::{Params, Value};
use persistent::Read;

#[derive(Copy, Clone)]
pub struct Log;

impl Key for Log { type Value = String; }

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
    Ok(Response::with((status::Ok, "SOS JS error logger server.")))
}

fn log_data(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<Log>>().unwrap();
    let log_path = arc.as_ref();

    let log_file = open_log(log_path);
    let mut log_writer = BufWriter::new(&log_file);

    let map = req.get_ref::<Params>().unwrap();

    // Try to find `error` key in the request data. If it is missing, we return 400: Bad request.
    match map.find(&["error"]) {
        Some(&Value::String(ref error)) => {
            let log_line = format!("[{datetime}] {error}\n", datetime=Local::now().to_string(), error=error);
            // Try to write into the LOG_FILE. When successful, return HTTP 200 Ok with the number
            // of bytes writtern. Otherwise, we return HTTP 500 Internal Server Error with the
            // reason.
            match log_writer.write(&log_line.into_bytes()) {
                Ok(bytes) => Ok(Response::with((status::Ok, format!("{}", bytes)))),
                Err(reason) => {
                    Ok(Response::with((status::InternalServerError, format!("{}", reason.description()))))
                }
            }
        },
        _ => Ok(Response::with((status::BadRequest, "Missing 'error' in POST data."))),
    }
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

    println!("Listening on {}", address);
    Iron::new(chain).http(address).unwrap();
}