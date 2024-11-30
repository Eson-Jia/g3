#!/bin/bash

curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.crt" -o /usr/bin/server_enc.crt && echo "Downloaded server_enc.crt"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.key" -o /usr/bin/server_enc.key && echo "Downloaded server_enc.key"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.crt" -o /usr/bin/server_sign.crt && echo "Downloaded server_sign.crt"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.key" -o /usr/bin/server_sign.key && echo "Downloaded server_sign.key"
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=chain-ca.crt" -o /usr/bin/chain-ca.crt && echo "Downloaded chain-ca.crt"
export SSL_CERT_FILE=/usr/bin/chain-ca.crt
# run g3proxy

/usr/bin/g3proxy -c /usr/bin/container.yaml --verbose
