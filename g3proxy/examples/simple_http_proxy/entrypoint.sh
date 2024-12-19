#!/bin/bash

curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.crt"  -s -o /usr/bin/server_enc.crt  -f  && echo "Downloaded server_enc.crt"  || { echo "Failed to download server_enc.crt"; exit 1; }
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.key"  -s -o /usr/bin/server_enc.key  -f  && echo "Downloaded server_enc.key"  || { echo "Failed to download server_enc.key"; exit 1; }
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.crt" -s -o /usr/bin/server_sign.crt -f  && echo "Downloaded server_sign.crt" || { echo "Failed to download server_sign.crt"; exit 1; }
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_sign.key" -s -o /usr/bin/server_sign.key -f  && echo "Downloaded server_sign.key" || { echo "Failed to download server_sign.key"; exit 1; }
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=chain-ca.crt"    -s -o /usr/bin/chain-ca.crt    -f  && echo "Downloaded chain-ca.crt"    || { echo "Failed to download chain-ca.crt"; exit 1; }
curl "http://nacos:8848/nacos/v1/cs/configs?group=gmssl&dataId=eq-rcd-zl-proxy" -s -o /usr/bin/container.yaml  -f  && echo "Downloaded container.yaml"
export SSL_CERT_FILE=/usr/bin/chain-ca.crt
# run g3proxy

/usr/bin/g3proxy -c /usr/bin/container.yaml --verbose
