#!/bin/bash
nacos=10.30.41.31


curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.crt" - /usr/bin/server_enc.crt
curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=server_enc.key" -o /usr/bin/server_enc.key
curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=sever_sign.crt" -o /usr/bin/sever_sign.crt
curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=sever_sign.key" -o /usr/bin/sever_sign.key
curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=subca.crt" -o /usr/bin/chain-ca.crt
curl "http://${nacos}:8848/nacos/v1/cs/configs?group=gmssl&dataId=ca.crt" >> /usr/bin/chain-ca.crt

export SSL_CERT_FILE=/usr/bin/chain-ca.crt
# run g3proxy
/usr/bin/g3proxy -c /usr/bin/ntls.yaml --verbose
