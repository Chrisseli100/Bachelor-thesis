# Bachelor-Thesis

This repository contains all relevant code for the **Webserver im Leistungsvergleich: Java vs. Rust** thesis. The documentation below is meant for running the client/servers on Linux, but the steps will be very similar on Windows/MacOS. 

To run larger test, use:
`sudo sysctl -w net.ipv4.tcp_tw_reuse=1`
This is necessary, because the system will otherwise run out of ports.

Systems with SE-Linux might need to turn it off:
`sudo setenforce 0`

## Java-Server

The tested java-server uses the gradle-toolchain and is is located at *java_testserver*. Changes to the threadpool size need to be made inside the code. The pool size is at *app/src/main/java/org/example/App.java* in line nine. The server listens to all incoming connections on port 6157.

To start the server:
`gradle run`

To compile the server into a jar:
`gradle clean build -x test`

## Rust-Server

The tested rust-server uses the cargo-toolchain and is located at *rust_testserver*. Changes to the threadpool size need to be made inside the code. The pool size is at *src/main.rs* in line nine. The server listens to all incoming connections on port 6157.

To start the server:
`cargo run` 
This will only run a slow version of the server for testing purposes.

To run the server optimised, compile it first:
`cargo build --release`

Then run the server with:
`./target/release/rust_testserver`

## Client

The tested client uses the cargo-toolchain and is located at *testclient*. The client is configured via the *configurationfile.txt*. The field *requestfile* loads the set testfile for the loadtest. All available txt-files can be used to test the server with, but only one file can be loaded per run. The field *send_incorrect_request* is only used for debugging purposes and should be disabled by default. It is used to send an incorrect formatted HTTP-request to the server, which the server needs to handle.

To start the client:
`cargo run`
This will only run a slow version of the client for testing purposes.

To run the client with optimised, compile it first:
`cargo build --release`

Then start the client with:
`./target/release/testclient`

The client will load the configured requestfile into memory and then wait, until the user starts the test. When the user starts the test, the client will send the entire list of requests to the running server and records the time. After finishing the test, the client will display the entire testduration on stdout and write all individual responsetimes into *results.csv*.
