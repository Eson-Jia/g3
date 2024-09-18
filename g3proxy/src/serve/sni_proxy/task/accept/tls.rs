/*
 * Copyright 2023 ByteDance and/or its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt};

use g3_dpi::parser::tls::{
    ClientHello, ExtensionType, HandshakeCoalescer, Record, RecordParseError,
};
use g3_types::net::{Host, TlsServerName, UpstreamAddr};

use crate::serve::{ServerTaskError, ServerTaskResult};

pub(super) async fn parse_request<R>(
    clt_r: &mut R,
    clt_r_buf: &mut BytesMut,
    port: u16,
    max_client_hello_size: u32,
) -> ServerTaskResult<UpstreamAddr>
where
    R: AsyncRead + Unpin,
{
    let mut handshake_coalescer = HandshakeCoalescer::new(max_client_hello_size);
    let mut record_offset = 0;
    loop {
        let mut record = match Record::parse(&clt_r_buf[record_offset..]) {
            Ok(r) => r,
            Err(RecordParseError::NeedMoreData(_)) => match clt_r.read_buf(clt_r_buf).await {
                Ok(0) => return Err(ServerTaskError::ClosedByClient),
                Ok(_) => continue,
                Err(e) => return Err(ServerTaskError::ClientTcpReadFailed(e)),
            },
            Err(_) => {
                return Err(ServerTaskError::InvalidClientProtocol(
                    "invalid tls client hello request",
                ));
            }
        };
        record_offset += record.encoded_len();

        // The Client Hello Message MUST be the first Handshake message
        match record.consume_handshake(&mut handshake_coalescer) {
            Ok(Some(handshake_msg)) => {
                let ch = handshake_msg.parse_client_hello().map_err(|_| {
                    ServerTaskError::InvalidClientProtocol("invalid tls client hello request")
                })?;
                return parse_sni(ch, port);
            }
            Ok(None) => match handshake_coalescer.parse_client_hello() {
                Ok(Some(ch)) => return parse_sni(ch, port),
                Ok(None) => {
                    if !record.consume_done() {
                        return Err(ServerTaskError::InvalidClientProtocol(
                            "partial fragmented tls client hello request",
                        ));
                    }
                }
                Err(_) => {
                    return Err(ServerTaskError::InvalidClientProtocol(
                        "invalid fragmented tls client hello request",
                    ));
                }
            },
            Err(_) => {
                return Err(ServerTaskError::InvalidClientProtocol(
                    "invalid tls client hello request",
                ));
            }
        }
    }
}

fn parse_sni(ch: ClientHello, port: u16) -> ServerTaskResult<UpstreamAddr> {
    match ch.get_ext(ExtensionType::ServerName) {
        Ok(Some(data)) => {
            let sni = TlsServerName::from_extension_value(data).map_err(|_| {
                ServerTaskError::InvalidClientProtocol(
                    "invalid server name in tls client hello message",
                )
            })?;
            Ok(UpstreamAddr::new(Host::from(sni), port))
        }
        Ok(None) => Err(ServerTaskError::InvalidClientProtocol(
            "no server name found in tls client hello message",
        )),
        Err(_) => Err(ServerTaskError::InvalidClientProtocol(
            "invalid extension in tls client hello request",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::sync::Arc;
    use tokio::io::Result;
    use tokio_util::io::StreamReader;

    #[tokio::test]
    async fn single_read() {
        let data: &[u8] = &[
            0x16, //
            0x03, 0x01, // TLS 1.0
            0x00, 0x65, // Fragment Length, 101
            0x01, // Handshake Type - ClientHello
            0x00, 0x00, 0x61, // Message Length, 97
            0x03, 0x03, // TLS 1.2
            0x74, 0x90, 0x65, 0xea, 0xbb, 0x00, 0x5d, 0xf8, 0xdf, 0xd6, 0xde, 0x04, 0xf8, 0xd3,
            0x69, 0x02, 0xf5, 0x8c, 0x82, 0x50, 0x7a, 0x40, 0xf6, 0xf3, 0xbb, 0x18, 0xc0, 0xac,
            0x4f, 0x55, 0x9a, 0xda, // Random data, 32 bytes
            0x20, // Session ID Length
            0x57, 0x5a, 0x8d, 0x9c, 0xa3, 0x8e, 0x16, 0xbd, 0xb6, 0x6c, 0xe7, 0x35, 0x62, 0x63,
            0x7f, 0x51, 0x5f, 0x6e, 0x97, 0xf7, 0xf9, 0x85, 0xad, 0xf0, 0x2d, 0x3a, 0x72, 0x9d,
            0x71, 0x0b, 0xe1, 0x32, // Session ID, 32 bytes
            0x00, 0x04, // Cipher Suites Length
            0x13, 0x02, 0x13, 0x01, // Cipher Suites
            0x01, // Compression Methods Length
            0x00, // Compression Methods
            0x00, 0x14, // Extensions Length, 20
            0x00, 0x00, // Extension Type - Server Name
            0x00, 0x10, // Extension Length, 16
            0x00, 0x0e, // Server Name List Length, 14
            0x00, // Server Name Type - Domain
            0x00, 0x0b, // Server Name Length, 11
            b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'n', b'e', b't',
        ];

        let content = b"test body\n";
        let stream = tokio_stream::iter(vec![Result::Ok(Bytes::from_static(content))]);
        let mut stream = StreamReader::new(stream);

        let mut clt_r_buf = BytesMut::from(data);

        let upstream = parse_request(&mut stream, &mut clt_r_buf, 443, 1 << 16)
            .await
            .unwrap();
        assert_eq!(
            upstream,
            UpstreamAddr::new(Host::Domain(Arc::from("example.net")), 443)
        );
    }

    #[tokio::test]
    async fn multi_read() {
        let data: &[u8] = &[
            0x16, //
            0x03, 0x01, // TLS 1.0
            0x00, 0x65, // Fragment Length, 101
        ];
        let data1: &[u8] = &[
            0x01, // Handshake Type - ClientHello
            0x00, 0x00, 0x61, // Message Length, 97
            0x03, 0x03, // TLS 1.2
            0x74, 0x90, 0x65, 0xea, 0xbb, 0x00, 0x5d, 0xf8, 0xdf, 0xd6, 0xde, 0x04, 0xf8, 0xd3,
            0x69, 0x02, 0xf5, 0x8c, 0x82, 0x50, 0x7a, 0x40, 0xf6, 0xf3, 0xbb, 0x18, 0xc0, 0xac,
            0x4f, 0x55, 0x9a, 0xda, // Random data, 32 bytes
            0x20, // Session ID Length
            0x57, 0x5a, 0x8d, 0x9c, 0xa3, 0x8e, 0x16, 0xbd, 0xb6, 0x6c, 0xe7, 0x35, 0x62, 0x63,
            0x7f, 0x51, 0x5f, 0x6e, 0x97, 0xf7, 0xf9, 0x85, 0xad, 0xf0, 0x2d, 0x3a, 0x72, 0x9d,
            0x71, 0x0b, 0xe1, 0x32, // Session ID, 32 bytes
            0x00, 0x04, // Cipher Suites Length
            0x13, 0x02, 0x13, 0x01, // Cipher Suites
            0x01, // Compression Methods Length
            0x00, // Compression Methods
        ];
        let data2: &[u8] = &[
            0x00, 0x14, // Extensions Length, 20
            0x00, 0x00, // Extension Type - Server Name
            0x00, 0x10, // Extension Length, 16
            0x00, 0x0e, // Server Name List Length, 14
            0x00, // Server Name Type - Domain
            0x00, 0x0b, // Server Name Length, 11
            b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'n', b'e', b't',
        ];

        let stream = tokio_stream::iter(vec![
            Result::Ok(Bytes::from_static(data1)),
            Result::Ok(Bytes::from_static(data2)),
        ]);
        let mut stream = StreamReader::new(stream);

        let mut clt_r_buf = BytesMut::from(data);

        let upstream = parse_request(&mut stream, &mut clt_r_buf, 443, 1 << 16)
            .await
            .unwrap();
        assert_eq!(
            upstream,
            UpstreamAddr::new(Host::Domain(Arc::from("example.net")), 443)
        );
    }

    #[tokio::test]
    async fn multi_record() {
        const RECORD_1_BYTES: &[u8] = &[
            0x16, 0x03, 0x01, 0x00, 0x64, 0x01, 0x00, 0x01, 0x8a, 0x03, 0x03, 0x02, 0x86, 0x70,
            0x33, 0x46, 0x28, 0x5f, 0x39, 0xc3, 0xf8, 0xa5, 0x3f, 0x3b, 0x39, 0x37, 0xb3, 0x68,
            0x9b, 0x3e, 0x21, 0x45, 0xff, 0x12, 0x74, 0x51, 0x7a, 0x27, 0xea, 0x73, 0x2f, 0x3a,
            0x6b, 0x20, 0x9c, 0x03, 0x35, 0x1a, 0xb3, 0x02, 0xbc, 0x68, 0x06, 0xc4, 0xad, 0x0d,
            0xce, 0xa9, 0x01, 0x0b, 0x1f, 0x24, 0x13, 0x6c, 0xb5, 0x73, 0xc2, 0x35, 0x77, 0xbd,
            0x74, 0x5e, 0x79, 0xec, 0xbf, 0x51, 0x00, 0x3a, 0x13, 0x02, 0x13, 0x03, 0x13, 0x01,
            0x13, 0x04, 0xc0, 0x2c, 0xcc, 0xa9, 0xc0, 0xad, 0xc0, 0x0a, 0xc0, 0x2b, 0xc0, 0xac,
            0xc0, 0x09, 0xc0, 0x30, 0xcc, 0xa8, 0xc0,
        ];
        const RECORD_2_BYTES: &[u8] = &[
            0x16, 0x03, 0x01, 0x00, 0x64, 0x14, 0xc0, 0x2f, 0xc0, 0x13, 0x00, 0x9d, 0xc0, 0x9d,
            0x00, 0x35, 0x00, 0x9c, 0xc0, 0x9c, 0x00, 0x2f, 0x00, 0x9f, 0xcc, 0xaa, 0xc0, 0x9f,
            0x00, 0x39, 0x00, 0x9e, 0xc0, 0x9e, 0x00, 0x33, 0x01, 0x00, 0x01, 0x07, 0x00, 0x05,
            0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x0b, 0x00,
            0x02, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x22, 0x00, 0x20, 0x04, 0x01, 0x08, 0x09, 0x08,
            0x04, 0x04, 0x03, 0x08, 0x07, 0x05, 0x01, 0x08, 0x0a, 0x08, 0x05, 0x05, 0x03, 0x08,
            0x08, 0x06, 0x01, 0x08, 0x0b, 0x08, 0x06, 0x06, 0x03, 0x02, 0x01, 0x02, 0x03, 0x00,
            0x17, 0x00, 0x00, 0x00, 0x10, 0x00, 0x0e,
        ];
        const RECORD_3_BYTES: &[u8] = &[
            0x16, 0x03, 0x01, 0x00, 0x64, 0x00, 0x0c, 0x02, 0x68, 0x32, 0x08, 0x68, 0x74, 0x74,
            0x70, 0x2f, 0x31, 0x2e, 0x31, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x11, 0x00, 0x00, 0x0e, 0x77, 0x77, 0x77, 0x2e, 0x67, 0x6f, 0x6f, 0x67, 0x6c,
            0x65, 0x2e, 0x63, 0x6f, 0x6d, 0x00, 0x1c, 0x00, 0x02, 0x40, 0x01, 0x00, 0x33, 0x00,
            0x6b, 0x00, 0x69, 0x00, 0x17, 0x00, 0x41, 0x04, 0xc1, 0x22, 0xc2, 0x9b, 0x8c, 0x56,
            0x55, 0xb6, 0x08, 0xd7, 0x4f, 0xdc, 0x56, 0xf2, 0xf6, 0xc7, 0x14, 0x5d, 0x0c, 0x65,
            0x6e, 0x9a, 0xb4, 0x55, 0x48, 0x60, 0x93, 0xfa, 0x4e, 0xdb, 0x53, 0x3e, 0x26, 0x7e,
            0xd2, 0xb3, 0x92, 0xe4, 0x35, 0xc3, 0x96,
        ];
        const RECORD_4_BYTES: &[u8] = &[
            0x16, 0x03, 0x01, 0x00, 0x62, 0xbb, 0x75, 0x13, 0x6d, 0xdf, 0x50, 0xc3, 0x8a, 0xd3,
            0xc3, 0xb5, 0x8a, 0x99, 0x32, 0x57, 0xad, 0x5d, 0xe9, 0x03, 0xb7, 0x07, 0xb1, 0x64,
            0x00, 0x1d, 0x00, 0x20, 0x0b, 0x8f, 0xf7, 0x47, 0x1b, 0x71, 0x67, 0x99, 0xfb, 0x54,
            0x76, 0xf1, 0x19, 0x64, 0x47, 0x61, 0xb3, 0x01, 0x8a, 0x90, 0x77, 0x19, 0xa7, 0x4c,
            0xbf, 0xd0, 0x17, 0x92, 0xc1, 0x25, 0x38, 0x35, 0x00, 0x0a, 0x00, 0x16, 0x00, 0x14,
            0x00, 0x17, 0x00, 0x18, 0x00, 0x19, 0x00, 0x1d, 0x00, 0x1e, 0x01, 0x00, 0x01, 0x01,
            0x01, 0x02, 0x01, 0x03, 0x01, 0x04, 0x00, 0x2b, 0x00, 0x09, 0x08, 0x03, 0x04, 0x03,
            0x03, 0x03, 0x02, 0x03, 0x01,
        ];

        let stream = tokio_stream::iter(vec![
            Result::Ok(Bytes::from_static(RECORD_1_BYTES)),
            Result::Ok(Bytes::from_static(RECORD_2_BYTES)),
            Result::Ok(Bytes::from_static(RECORD_3_BYTES)),
            Result::Ok(Bytes::from_static(RECORD_4_BYTES)),
        ]);
        let mut stream = StreamReader::new(stream);

        let mut clt_r_buf = BytesMut::new();

        let upstream = parse_request(&mut stream, &mut clt_r_buf, 443, 1 << 16)
            .await
            .unwrap();
        assert_eq!(
            upstream,
            UpstreamAddr::new(Host::Domain(Arc::from("www.google.com")), 443)
        );
    }
}
