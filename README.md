# rust-http-logger
[![Crates.io]
(https://img.shields.io/crates/v/rust-http-logger.svg)](https://crates.io/crates/rust-http-logger)

HTTP Server for logging from client JS to a server (into a file).

The server accepts JSON or HTML Form POST data:

**POST**:

    error=my data&info=another data&debug=debug data
    
**JSON** (don't forget to add Content-type: application/json):
    
    {
      "error":"Some error happened: TRACEBACK ... ", 
      "info":"This user is suspicious", 
      "warning":"Oh, this happened, beware!", 
      "debug":"NOOO BUGs, BUGs everywhere!"
    }

## Usage

    rust-http-logger localhost:3000 api.log

Now the server accepts incomming connections on port 3000 on your machine and logs the data onto `api.log`. To test this
you can use `curl`.

    curl -X POST -d "error=my data" http://localhost:3000/log/
    
You will get a JSON response back, like this:

    {"success":{"error":"51"},"errors":{}}
    
This means we wrote 51B to the log file successfully under `error` log level. Errors (if any) will be shown in "errors"
key of the JSON structure.

The `api.log` file should contain:

    [2016-08-06 13:34:04.091372 +02:00] [ERROR] my data

    
## Install
Best way is to install `rust` and compile the binary yourself. There are some pre-build binaries in the `Releases` section.
 
## Performance
On my machine (Macbook Air) using `wrk`, about 13k requests/s with 1000 concurrent connections.

    wrk -d60s -c1000 -t100 -s wrk-script.lua http://localhost:3000/log/
    Running 1m test @ http://localhost:3000/log/
      100 threads and 1000 connections
      Thread Stats   Avg      Stdev     Max   +/- Stdev
        Latency     2.36ms    1.14ms  87.90ms   95.13%
        Req/Sec     3.38k     1.03k    5.64k    60.32%
      807848 requests in 1.00m, 138.68MB read
      Socket errors: connect 0, read 2433, write 0, timeout 0
    Requests/sec:  13440.71
    Transfer/sec:      2.31MB
    
wrk-script.lua:

    wrk.scheme = "http"
    wrk.host = "localhost"
    wrk.port = 3000
    wrk.method = "POST"
    wrk.path = "/log/"
    wrk.headers["Content-Type"] = "application/json"
    
    wrk.body = [[{
        "error":"OHH performance",
        "info":"much information, very wow",
        "warning":"so warn, much very wow",
        "debug":"oh the debugs, so many debugs"
    }]]
