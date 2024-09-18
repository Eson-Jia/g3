/*
 * Copyright 2024 ByteDance and/or its affiliates.
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

use g3_types::net::TlsServerName;

use crate::parser::tls::{ExtensionType, HandshakeCoalescer, Record};

const RECORD_1_BYTES: &[u8] = &[
    0x16, 0x03, 0x01, 0x00, 0x64, 0x01, 0x00, 0x01, 0x8a, 0x03, 0x03, 0x02, 0x86, 0x70, 0x33, 0x46,
    0x28, 0x5f, 0x39, 0xc3, 0xf8, 0xa5, 0x3f, 0x3b, 0x39, 0x37, 0xb3, 0x68, 0x9b, 0x3e, 0x21, 0x45,
    0xff, 0x12, 0x74, 0x51, 0x7a, 0x27, 0xea, 0x73, 0x2f, 0x3a, 0x6b, 0x20, 0x9c, 0x03, 0x35, 0x1a,
    0xb3, 0x02, 0xbc, 0x68, 0x06, 0xc4, 0xad, 0x0d, 0xce, 0xa9, 0x01, 0x0b, 0x1f, 0x24, 0x13, 0x6c,
    0xb5, 0x73, 0xc2, 0x35, 0x77, 0xbd, 0x74, 0x5e, 0x79, 0xec, 0xbf, 0x51, 0x00, 0x3a, 0x13, 0x02,
    0x13, 0x03, 0x13, 0x01, 0x13, 0x04, 0xc0, 0x2c, 0xcc, 0xa9, 0xc0, 0xad, 0xc0, 0x0a, 0xc0, 0x2b,
    0xc0, 0xac, 0xc0, 0x09, 0xc0, 0x30, 0xcc, 0xa8, 0xc0,
];
const RECORD_2_BYTES: &[u8] = &[
    0x16, 0x03, 0x01, 0x00, 0x64, 0x14, 0xc0, 0x2f, 0xc0, 0x13, 0x00, 0x9d, 0xc0, 0x9d, 0x00, 0x35,
    0x00, 0x9c, 0xc0, 0x9c, 0x00, 0x2f, 0x00, 0x9f, 0xcc, 0xaa, 0xc0, 0x9f, 0x00, 0x39, 0x00, 0x9e,
    0xc0, 0x9e, 0x00, 0x33, 0x01, 0x00, 0x01, 0x07, 0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x02, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x22, 0x00,
    0x20, 0x04, 0x01, 0x08, 0x09, 0x08, 0x04, 0x04, 0x03, 0x08, 0x07, 0x05, 0x01, 0x08, 0x0a, 0x08,
    0x05, 0x05, 0x03, 0x08, 0x08, 0x06, 0x01, 0x08, 0x0b, 0x08, 0x06, 0x06, 0x03, 0x02, 0x01, 0x02,
    0x03, 0x00, 0x17, 0x00, 0x00, 0x00, 0x10, 0x00, 0x0e,
];
const RECORD_3_BYTES: &[u8] = &[
    0x16, 0x03, 0x01, 0x00, 0x64, 0x00, 0x0c, 0x02, 0x68, 0x32, 0x08, 0x68, 0x74, 0x74, 0x70, 0x2f,
    0x31, 0x2e, 0x31, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x13, 0x00, 0x11, 0x00, 0x00,
    0x0e, 0x77, 0x77, 0x77, 0x2e, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x2e, 0x63, 0x6f, 0x6d, 0x00,
    0x1c, 0x00, 0x02, 0x40, 0x01, 0x00, 0x33, 0x00, 0x6b, 0x00, 0x69, 0x00, 0x17, 0x00, 0x41, 0x04,
    0xc1, 0x22, 0xc2, 0x9b, 0x8c, 0x56, 0x55, 0xb6, 0x08, 0xd7, 0x4f, 0xdc, 0x56, 0xf2, 0xf6, 0xc7,
    0x14, 0x5d, 0x0c, 0x65, 0x6e, 0x9a, 0xb4, 0x55, 0x48, 0x60, 0x93, 0xfa, 0x4e, 0xdb, 0x53, 0x3e,
    0x26, 0x7e, 0xd2, 0xb3, 0x92, 0xe4, 0x35, 0xc3, 0x96,
];
const RECORD_4_BYTES: &[u8] = &[
    0x16, 0x03, 0x01, 0x00, 0x62, 0xbb, 0x75, 0x13, 0x6d, 0xdf, 0x50, 0xc3, 0x8a, 0xd3, 0xc3, 0xb5,
    0x8a, 0x99, 0x32, 0x57, 0xad, 0x5d, 0xe9, 0x03, 0xb7, 0x07, 0xb1, 0x64, 0x00, 0x1d, 0x00, 0x20,
    0x0b, 0x8f, 0xf7, 0x47, 0x1b, 0x71, 0x67, 0x99, 0xfb, 0x54, 0x76, 0xf1, 0x19, 0x64, 0x47, 0x61,
    0xb3, 0x01, 0x8a, 0x90, 0x77, 0x19, 0xa7, 0x4c, 0xbf, 0xd0, 0x17, 0x92, 0xc1, 0x25, 0x38, 0x35,
    0x00, 0x0a, 0x00, 0x16, 0x00, 0x14, 0x00, 0x17, 0x00, 0x18, 0x00, 0x19, 0x00, 0x1d, 0x00, 0x1e,
    0x01, 0x00, 0x01, 0x01, 0x01, 0x02, 0x01, 0x03, 0x01, 0x04, 0x00, 0x2b, 0x00, 0x09, 0x08, 0x03,
    0x04, 0x03, 0x03, 0x03, 0x02, 0x03, 0x01,
];

#[test]
fn sni() {
    // check full data
    let mut handshake_coalescer = HandshakeCoalescer::default();

    let mut record1 = Record::parse(RECORD_1_BYTES).unwrap();
    let handshake_msg = record1.consume_handshake(&mut handshake_coalescer).unwrap();
    assert!(handshake_msg.is_none());
    assert!(record1.consume_done());
    let client_hello = handshake_coalescer.parse_client_hello().unwrap();
    assert!(client_hello.is_none());

    let mut record2 = Record::parse(RECORD_2_BYTES).unwrap();
    let handshake_msg = record2.consume_handshake(&mut handshake_coalescer).unwrap();
    assert!(handshake_msg.is_none());
    assert!(record2.consume_done());
    let client_hello = handshake_coalescer.parse_client_hello().unwrap();
    assert!(client_hello.is_none());

    let mut record3 = Record::parse(RECORD_3_BYTES).unwrap();
    let handshake_msg = record3.consume_handshake(&mut handshake_coalescer).unwrap();
    assert!(handshake_msg.is_none());
    assert!(record3.consume_done());
    let client_hello = handshake_coalescer.parse_client_hello().unwrap();
    assert!(client_hello.is_none());

    let mut record4 = Record::parse(RECORD_4_BYTES).unwrap();
    let handshake_msg = record4.consume_handshake(&mut handshake_coalescer).unwrap();
    assert!(handshake_msg.is_none());
    assert!(record4.consume_done());
    let client_hello = handshake_coalescer.parse_client_hello().unwrap().unwrap();
    let sni_bytes = client_hello
        .get_ext(ExtensionType::ServerName)
        .unwrap()
        .unwrap();
    let sni = TlsServerName::from_extension_value(sni_bytes).unwrap();
    assert_eq!(sni.as_ref(), "www.google.com");
}
