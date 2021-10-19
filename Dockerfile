FROM rust:slim

RUN apt-get update && apt-get install -y \
    build-essential autoconf automake libtool m4 \
    ffmpeg \
    youtube-dl \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR "/parrot"

# Cache cargo build dependencies by creating a dummy source
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml ./
RUN cargo build --release

COPY ./ ./
RUN cargo build --release

CMD ["cargo", "run", "--release"]