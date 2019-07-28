FROM node:alpine as frontend

RUN yarn global add elm elm-test

COPY ./frontend/elm.json ./elm.json
COPY ./frontend/src ./src
COPY ./frontend/tests ./tests

RUN elm make src/Main.elm


FROM ekidd/rust-musl-builder:stable as backend

# create new cargo project
RUN USER=rust cargo init --bin
# copy build config
COPY --chown=rust ./backend/Cargo.lock ./Cargo.lock
COPY --chown=rust ./backend/Cargo.toml ./Cargo.toml
# build to cache dependencies
RUN cargo build --release
# delete build cache to prevent caching issues later on
RUN rm -r ./target/x86_64-unknown-linux-musl/release/.fingerprint/gitache-*

COPY ./backend/static ./static
COPY ./backend/src ./src
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

COPY --from=backend /home/rust/src/target/x86_64-unknown-linux-musl/release/gitache /
COPY --from=frontend /index.html /public/

ENTRYPOINT ["/gitache"]