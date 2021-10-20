FROM rust:slim-bullseye as build

RUN apt-get update && apt-get install -y \
    build-essential autoconf automake libtool m4 \
    libssl-dev pkg-config

WORKDIR "/parrot"

# Cache cargo build dependencies by creating a dummy source
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml ./
RUN cargo build --release

COPY . .
RUN cargo build --release

# Our final base
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ffmpeg youtube-dl

# Copy the build artifact from the build stage
COPY --from=build /parrot/target/release/parrot .

# Run parrot's binary
CMD ["./parrot"]
