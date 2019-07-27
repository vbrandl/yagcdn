FROM ekidd/rust-musl-builder:stable as builder

# create new cargo project
RUN USER=rust cargo init --bin
# copy build config
COPY --chown=rust ./Cargo.lock ./Cargo.lock
COPY --chown=rust ./Cargo.toml ./Cargo.toml
# build to cache dependencies
RUN cargo build --release
# delete build cache to prevent caching issues later on
RUN rm -r ./target/x86_64-unknown-linux-musl/release/.fingerprint/gitache-*

COPY ./static ./static
COPY ./src ./src
# build source code
RUN cargo build --release


# create /etc/password for rootless scratch container
FROM alpine:latest as user_builder
RUN USER=root adduser -D -u 10001 dummy

FROM scratch

# copy certificates
COPY --from=linuxkit/ca-certificates:v0.7 / /
COPY --from=user_builder /etc/passwd /etc/passwd
USER dummy

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/gitache /

ENTRYPOINT ["/gitache"]
