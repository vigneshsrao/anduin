# Anduin

Simple HTTP server primarily written for the sake of getting familiar with Rust.

This server handles GET requests by trying to read the given URI as a file and returning 404 if not found. There are no path traversal checks implemented yet.
POST requests are just printed out to standard output.
It also tries to go into a chroot "sandbox" by default just because... well why not ?

I use this when I want a simple server to test stuff which also handles POST requests by dumping them onto stdout. Very useful for testing exploits :D.

# Usage

* Install rust and cargo
* `cargo build --release` in the project root
* `cargo run --release -- --help`

```
Usage: anduin [OPTIONS]

Options are:

-i, --host <IP>         The IP address on which to host the server [default: 0.0.0.0]
-p, --port <PORT>       The port on which to run server [default: 8000]
--no-sandbox            Run the server without the sandbox
-h, --help              Print this help information and exit
```



