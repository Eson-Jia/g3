#!/bin/bash

curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.crt"  -s -o /usr/bin/server_enc.crt    && echo "Downloaded server_enc.crt"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.key"  -s -o /usr/bin/server_enc.key    && echo "Downloaded server_enc.key"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.crt" -s -o /usr/bin/server_sign.crt   && echo "Downloaded server_sign.crt"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.key" -s -o /usr/bin/server_sign.key   && echo "Downloaded server_sign.key"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=chain-ca.crt"    -s -o /usr/bin/chain-ca.crt      && echo "Downloaded chain-ca.crt"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=eq-rcd-zl-proxy" -s -o /usr/bin/container.yaml -f && echo "Downloaded container.yaml"
export SSL_CERT_FILE=/usr/bin/chain-ca.crt
# run g3proxy

/usr/bin/g3proxy -c /usr/bin/container.yaml --verbose
