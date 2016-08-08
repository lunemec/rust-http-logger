use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};

use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;

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

pub fn help(_: &mut Request) -> IronResult<Response> {
    let content_type = Header(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
    Ok(Response::with((status::Ok, HELP_MSG, content_type)))
}