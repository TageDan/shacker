FROM rust:slim-bookworm AS build

WORKDIR /tmp/sshack

RUN apt-get update && apt-get install -y libsqlite3-dev

# Copy only Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file so cargo can build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only - this layer is cached unless Cargo files change
RUN cargo build --release

# Remove the dummy build artifacts (but keep compiled dependencies)
RUN rm -rf src target/release/deps/sshack* target/release/sshack*

COPY ./ ./

RUN cargo build --release

RUN mkdir -p /home/sshack/.config/sshack/ && \
mkdir -p home/sshack/.config/sshack/files

COPY ./themes /home/sshack/.config/sshack/themes
COPY ./config.toml /home/sshack/.config/sshack/

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-dev

RUN useradd -ms /bin/bash sshack

USER sshack

COPY --from=build --chown=sshack:sshack /tmp/sshack/target/release/sshack ./

COPY --from=build /home/sshack/.config/sshack /home/sshack/.config/sshack

CMD ["./sshack", "run"]

