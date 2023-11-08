## Horcrust
Split your precious secret in shares and send them to remote servers. Easily recover your secret when you need it.


### Running the system

I’ve provided a Dockerfile to build the servers and a docker-compose to ease testing. If you want to run it, you will need either docker and docker-compose or rust-toolchain + cargo.

```
docker-compose up

```

or

```
cargo run --bin server -- --port 9091 -s 127.0.0.1:9091 -s 127.0.0.1:9092
cargo run --bin server -- --port 9092 -s 127.0.0.1:9091 -s 127.0.0.1:9092

```

To run the client, I’ve provided a Dockerfile-client file:

```jsx
# build:
docker build -f Dockerfile-client -t horcrust-client:latest .
# run it:
docker run -it --rm horcrust-client --help
docker run --network=host -it --rm horcrust-client -s 127.0.0.1:9091 -s 127.0.0.1:9092 store-secret 123 323
docker run --network=host -it --rm horcrust-client  -s 127.0.0.1:9091 -s 127.0.0.1:9092 retrieve-secret 123

```

To run the client through cargo:

```
cargo run --bin client -- -s 127.0.0.1:9091 -s 127.0.0.1:9092 store-secret 123 323
cargo run --bin client -- -s 127.0.0.1:9091 -s 127.0.0.1:9092 retrieve-secret 123
cargo run --bin client -- --help
Create shares out of your secret and stores them to distributed servers. Allows you to safely recover your secret from the shares on a later moment

Usage: client --servers <SERVERS> <COMMAND>

Commands:
  store-secret     
  retrieve-secret  
  help             Print this message or the help of the given subcommand(s)

Options:
  -s, --servers <SERVERS>  a list of servers to store your secret. Please provide at least 2 servers
  -h, --help               Print help
  -V, --version            Print version

```
