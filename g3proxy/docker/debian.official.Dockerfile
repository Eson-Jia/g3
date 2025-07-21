FROM g3proxy:official

COPY g3proxy/examples/simple_http_proxy/certificate.crt /usr/bin/certificate.crt
COPY g3proxy/examples/simple_http_proxy/private.key /usr/bin/private.key
COPY g3proxy/examples/simple_http_proxy/config.yaml /usr/bin/config.yaml

ENTRYPOINT ["/usr/bin/g3proxy"]
CMD ["-c", "/usr/bin/config.yaml", "--verbose"]