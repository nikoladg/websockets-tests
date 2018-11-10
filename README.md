# websockets-tests

build with `cargo +nightly build`
run with `cargo +nightly run`

* `echo` is a websocket echo server accessible via `ws://localhost:8080/wsecho/`
* `forwarder` forwards websocket messages to `echo` and is accessible via `ws://localhost:8081/wsforwarder/`
* `sender` sends command-line input as text websocket messages to `forwarder` (it can also be modified to send directly to `echo`)
