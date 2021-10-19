FROM rust:slim as build

RUN apt-get update && apt-get install -y \
    build-essential autoconf automake libtool m4 \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin parrot
WORKDIR "/parrot"

# Cache cargo build dependencies by creating a dummy source
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml ./
RUN cargo build --release

COPY . .
RUN rm ./target/release/deps/parrot*
RUN cargo build --release

# our final base
FROM debian:buster-slim

RUN apt-get update && apt-get install -y ffmpeg youtube-dl

# copy the build artifact from the build stage
COPY --from=build /parrot/target/release/parrot .
COPY --from=build /parrot/.env .

# set the startup command to run your binary
CMD ["./parrot"]
