# netflow_collector

Simple Netflow traffic listener. Stores the Netflow data in the JSON format.

## Example
```
cargo run -- --help
Netflow colector 0.1.0
Netflow collector

USAGE:
    netflow_collector [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <bind_address>    Address to bind. Default: 127.0.0.1
    -o, --output <output>        Name of the file the JSON output is stored in. Will be created if does not exists.
    -p, --port <port>            Listening port. Default: 2055
    ```
