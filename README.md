# multi-chat

## How to run

You will need these dependencies

1. [Rust toolchain](https://rustup.rs/), To compile the code
2. [Redis](https://redis.io/), This allows multiple connections to consume the chat websocket
3. You will need a google api credentials to access youtube chat

Create a folder call `tmp`, credentials will be placed there.

```bash
mkdir tmp
```

Once you downloaded the oauth2 credentials file move them into the `tmp` folder

for example

```bash
mv  ~/Downloads/client_secret_asd123.apps.googleusercontent.com.json tmp/credentials.json
```

next start the redis server

```bash
redis-server
```

Then run the program

```bash
# compile
cargo build --release
# run the program
./target/release/chat-rs all
```
