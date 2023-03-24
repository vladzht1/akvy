# Akvy - simple HTTP API stress-test util

## Usage

    ./akvy -u http://localhost:5000/api/users -r 1000 -m 10000

    Target URL: http://localhost:5000/api/users
    Requests per second: 1000

    // Use can wait for all requests to be sent or...
    ^C        // Ctrl + C for stop the application

    Elapsed:             3.50s
    Requests:            350
     - Success:          350
     - Errors:           0
    Percent of errors:   0.00%
    Response time:
    - Min:               0ms
    - Max:               11ms
    - Average:           1ms

## Compile youself

#### [Rust](https://www.rust-lang.org) must be installed.
    // Execute this commands in the project directory
    cargo build --release

##### The binary file will be located in the directory

    ../target/release
