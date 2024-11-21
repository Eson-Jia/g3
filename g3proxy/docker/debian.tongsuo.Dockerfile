FROM rust:bookworm AS builder
WORKDIR /usr/src/g3
COPY . .
RUN apt-get update && apt-get install -y libclang-dev cmake capnproto
RUN cargo build --profile release-lto \
 --no-default-features --features vendored-tongsuo,rustls-ring,quic,vendored-c-ares,hickory \
 -p g3proxy -p g3proxy-ctl

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY g3proxy/examples/simple_http_proxy/ /usr/bin/
COPY g3proxy/examples/simple_http_proxy/ /usr/local/share/ca-certificates/
RUN update-ca-certificates
COPY --from=builder /usr/src/g3/target/release-lto/g3proxy /usr/bin/g3proxy
COPY --from=builder /usr/src/g3/target/release-lto/g3proxy-ctl /usr/bin/g3proxy-ctl
#g3proxy --config-file ntls.yaml --verbose
ENTRYPOINT ["/usr/bin/g3proxy"]
CMD ["--config-file", "/usr/bin/ntls.yaml", "--verbose"]
