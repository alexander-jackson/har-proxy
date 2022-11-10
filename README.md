# `har-proxy`

Proxy server backed by a HAR (HTML archive) file, replaying requests from a previous session.

## Installation

You'll need a Rust toolchain to build and install this project. The following should work on MacOS:

```bash
# Install `rustup`
brew install rustup-init

# Install the stable version of Rust
rustup update stable

# Install `har-proxy`
cargo install --path .
```

## Usage

First you'll need a HAR file to replay requests from. This can be done by
opening the network tab in Chrome and moving around the website, before
right-clicking a request and selecting "Save all as HAR with content".

Once you've done this, you can run `har-proxy --from <har_file>`. You may also
want to optionally change the `port` and `prefix` arguments depending on the
port your server is normally on and whether it is behind a reverse proxy.
