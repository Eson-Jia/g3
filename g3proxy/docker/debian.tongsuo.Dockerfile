FROM rust:bookworm AS builder
WORKDIR /usr/src/g3
COPY . .
RUN apt-get update && apt-get install -y libclang-dev cmake capnproto
RUN cargo build --profile release-lto \
 --no-default-features --features vendored-tongsuo,rustls-ring,quic,vendored-c-ares,hickory \
 -p g3proxy

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl vim lsof tcpdump procps && rm -rf /var/lib/apt/lists/*

COPY g3proxy/examples/simple_http_proxy/certificate.crt /usr/bin/certificate.crt
COPY g3proxy/examples/simple_http_proxy/private.key /usr/bin/private.key
COPY g3proxy/examples/simple_http_proxy/ntls.yaml /usr/bin/ntls.yaml
COPY g3proxy/examples/simple_http_proxy/container.yaml /usr/bin/container.yaml
COPY --from=builder /usr/src/g3/target/release-lto/g3proxy /usr/bin/g3proxy

COPY g3proxy/examples/simple_http_proxy/entrypoint.sh /usr/bin/entrypoint.sh
RUN chmod +x /usr/bin/entrypoint.sh
ENTRYPOINT ["/usr/bin/entrypoint.sh"]
