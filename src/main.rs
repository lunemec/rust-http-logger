extern crate chrono;
extern crate clap;
extern crate hyper;
extern crate iron;
extern crate params;
extern crate persistent;
extern crate router;
extern crate rustc_serialize;

use std::error::Error;
use std::fmt::{self, Debug};
use std::sync::mpsc::channel;

use clap::{Arg, App};

use iron::AfterMiddleware;
use iron::prelude::*;
use iron::status;

use persistent::Write;
use router::Router;

mod global;
mod handlers;
mod writer;

struct ErrorRecover;

#[derive(Debug)]
struct StringError(String);

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
    let log_path = matches.value_of("LOG").unwrap().to_string();
    let serve_help = matches.is_present("serve_help");

    let (tx, rx) = channel::<String>();
    let mut router = Router::new();

    router.post("/log/", handlers::log_data);

    if serve_help {
        router.any("/", handlers::help);
    } else {
        router.any("/", |_: &mut Request| {Ok(Response::with(status::Forbidden))});
    }

    let mut chain = Chain::new(router);
    chain.link(Write::<global::DataSender>::both(tx));
    chain.link_after(ErrorRecover);

    writer::spawn_writer(log_path, rx);
    println!("Listening on {}", address);
    Iron::new(chain).http(address).unwrap();
}