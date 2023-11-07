# for the server.
FROM rust:1.72 as builder
WORKDIR /usr/src/myapp
COPY . .
# statically linking everything
RUN rustup target add x86_64-unknown-linux-musl
ARG CARGO_PARAMS
ARG GIT_COMMIT
ARG GIT_BRANCH
ARG IMAGE_NAME
RUN apt-get update && apt-get install -y protobuf-compiler
RUN echo "Running cargo build with params: $CARGO_PARAMS" && cargo build --target=x86_64-unknown-linux-musl --release --bin horcrust-server $CARGO_PARAMS

FROM debian:buster-slim
COPY --from=builder /usr/src/myapp/target/x86_64-unknown-linux-musl/release/horcrust-server /sbin/horcrust-server
ENV HORCRUST_LOG info
ENV GIT_COMMIT=$GIT_COMMIT
ENV GIT_BRANCH=$GIT_BRANCH
ENV CARGO_PARAMS=$CARGO_PARAMS
RUN echo "{\"rev\":\"$GIT_COMMIT\",\"branch\":\"${GIT_BRANCH}\",\"cargo-params\":\"${CARGO_PARAMS}\" }" > /buildinfo.json

ENTRYPOINT ["/sbin/horcrust-server"]

