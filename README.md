# rust-http-logger
HTTP Server for logging from client JS to a server (into a file).

The server accepts JSON or HTML Form POST data:

**POST**:

    error=my data&info=another data&debug=debug data
    
**JSON**:

    {
      "error":"Some error happened: TRACEBACK ... ", 
      "info":"This user is suspicious", 
      "warning":"Oh, this happened, beware!", 
      "debug":"NOOO BUGs, BUGs everywhere!"
    }

## Usage

    rust-http-logger localhost:3000 api.log --api_help

Now the server accepts incomming connections on port 3000 on your machine and logs the data onto `api.log`. To test this
you can use `curl`.

    curl -X POST -d "error=my data" http://localhost:3000/log/
    
You will get a JSON response back, like this:

    {"success":{"error":"51"},"errors":{}}
    
This means we wrote 51B to the log file successfully under `error` log level. Errors (if any) will be shown in "errors"
key of the JSON structure.

The `api.log` file should contain:

    [2016-08-06 13:34:04.091372 +02:00][ERROR] my data

    
## Install
Best way is to install `rust` and compile the binary yourself. There are some pre-build binaries in the `Releases` section. 