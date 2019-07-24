FROM ekidd/rust-musl-builder:stable as builder

# RUN adduser -D hoc
RUN useradd -U 10001 dummy

# create new cargo project
RUN USER=rust cargo init --bin
# copy build config
COPY --chown=rust ./Cargo.lock ./Cargo.lock
COPY --chown=rust ./Cargo.toml ./Cargo.toml
# build to cache dependencies
RUN cargo build --release
# delete build cache to prevent caching issues later on
RUN rm -r ./target/x86_64-unknown-linux-musl/release/.fingerprint/gitache-*

COPY ./src ./src
# build source code
RUN cargo build --release

FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
USER dummy

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/gitache /

ENTRYPOINT ["/gitache"]
