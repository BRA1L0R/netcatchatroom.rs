# NetCat chatroom

Simple netcat chatroom with rate limiting capabilities

## Todo

- [x] Rate limiter
- [x] IP hash instead of ip
- [x] Broadcasting client connection and disconnection on to the clients too
- [ ] Move run code from main to lib.rs keeping only basic logic in main.rs
- [ ] Extensive unit testing and integration testing

## How do I run it?

```
cargo build
```

The compiled executable will be available in the target directory.

The server automatically lisens on all ips on port 6161. Will have to configure command line options to change this port and/or ip to bind on.

## How do I connect to it?

Once the server is running you can open any linux shell, where the only requirement is to have netcat installed (which a lot of distros already come with preinstalled)

```
nc <ip> <port>
```
