FROM node:alpine as frontend

# install envsubst
RUN apk add -U --upgrade --no-cache gettext

RUN yarn global add elm uglify-js

COPY ./frontend/build.sh ./build.sh
COPY ./frontend/assets ./assets
COPY ./frontend/template.html ./template.html
COPY ./frontend/elm.json ./elm.json
COPY ./frontend/src ./src
# COPY ./frontend/tests ./tests

RUN ./build.sh

FROM ekidd/rust-musl-builder:stable as backend

# create new cargo project
RUN USER=rust cargo new --bin yagcdn
RUN USER=rust cargo new --lib time-cache
# copy build config
COPY --chown=rust ./backend/Cargo.lock ./yagcdn/Cargo.lock
COPY --chown=rust ./backend/Cargo.toml ./yagcdn/Cargo.toml
COPY --chown=rust ./time-cache/Cargo.toml ./time-cache/Cargo.toml

WORKDIR /home/rust/src/yagcdn
# build to cache dependencies
# RUN cargo build --release
# delete build cache to prevent caching issues later on
# RUN rm -r ./target/x86_64-unknown-linux-musl/release/.fingerprint/yagcdn*
# RUN rm -r ./target/x86_64-unknown-linux-musl/release/.fingerprint/time-cache-*

COPY ./backend/static ./static
COPY ./backend/src ./src
COPY ./time-cache/src ../time-cache/src
# build source code

RUN         --mount=type=cache,target=../../usr/local/cargo/registry \
            --mount=type=cache,target=target \
            cargo build --release

# create /etc/password for rootless scratch container
FROM alpine:latest as user_builder
RUN USER=root adduser -D -u 10001 dummy

FROM scratch

# copy certificates
COPY --from=linuxkit/ca-certificates:v0.7 / /
COPY --from=user_builder /etc/passwd /etc/passwd
USER dummy

COPY --from=backend /home/rust/src/yagcdn/target/x86_64-unknown-linux-musl/release/yagcdn /
COPY --from=frontend /output/index.html /public/index.html
COPY --from=frontend /output/scripts /public/scripts
COPY --from=frontend /output/assets /public/assets

ENTRYPOINT ["/yagcdn"]
