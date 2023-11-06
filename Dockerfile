# for the server.
FROM rust:1 as builder
WORKDIR /usr/src/myapp
COPY . .
ARG CARGO_PARAMS
ARG GIT_COMMIT
ARG GIT_BRANCH
ARG IMAGE_NAME

RUN echo "Running cargo build with params: $CARGO_PARAMS" && cargo build --release $CARGO_PARAMS

FROM debian:buster-slim
COPY --from=builder /usr/src/myapp/target/release/server /sbin/horcrust-server
RUN apt-get update && apt-get install bash
ENV HORCRUST_LOG info
ENV GIT_COMMIT=$GIT_COMMIT
ENV GIT_BRANCH=$GIT_BRANCH
ENV CARGO_PARAMS=$CARGO_PARAMS
RUN echo "{\"rev\":\"$GIT_COMMIT\",\"branch\":\"${GIT_BRANCH}\",\"cargo-params\":\"${CARGO_PARAMS}\" }" > /buildinfo.json

ENTRYPOINT ["/sbin/horcrust-server"]
