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

use crate::parser::quic::{HandshakeCoalescer, InitialPacket};
use crate::parser::tls::ExtensionType;

const PACKET1_BYTES: &[u8] = &[
    0xc7, 0x00, 0x00, 0x00, 0x01, 0x08, 0xc8, 0xce, 0x86, 0x5d, 0x32, 0xec, 0x58, 0x20, 0x00, 0x00,
    0x44, 0xd0, 0xb0, 0xbb, 0x69, 0xdd, 0x6d, 0xe2, 0x93, 0xf6, 0x8d, 0x81, 0xc9, 0x11, 0x77, 0x0f,
    0xe7, 0x10, 0xa8, 0x4b, 0xd3, 0x85, 0x25, 0x7b, 0xcb, 0xb4, 0xaa, 0x7c, 0x31, 0x2b, 0x95, 0x8f,
    0xae, 0x85, 0x68, 0x12, 0xfd, 0x6e, 0xc8, 0x39, 0xd1, 0x4b, 0x9b, 0x91, 0x60, 0x17, 0x0f, 0x59,
    0x94, 0xb2, 0xe3, 0xac, 0x2d, 0xc7, 0xee, 0x71, 0x27, 0x7e, 0x48, 0x46, 0x24, 0xe8, 0xc4, 0x79,
    0xb3, 0xf1, 0x1b, 0xd3, 0x9f, 0xc7, 0x39, 0xa9, 0xb0, 0xa0, 0xaa, 0xa4, 0xcd, 0x0f, 0xc1, 0x4f,
    0x41, 0xbd, 0xe7, 0xf2, 0x3c, 0xc3, 0xa9, 0x15, 0xc1, 0x33, 0x45, 0x26, 0x6e, 0xb1, 0x2c, 0xeb,
    0xfa, 0x1a, 0xf3, 0xdb, 0x01, 0x9c, 0x67, 0x28, 0xec, 0x07, 0x20, 0x08, 0x33, 0xd1, 0xce, 0x7c,
    0xac, 0xf2, 0xe7, 0x9d, 0xbb, 0x68, 0x42, 0x05, 0x95, 0x98, 0xd2, 0xd6, 0x98, 0x3c, 0x73, 0xc9,
    0xf6, 0x81, 0xf3, 0x41, 0x9e, 0x6b, 0xe7, 0x27, 0x81, 0xfe, 0x53, 0x1e, 0x34, 0x63, 0x21, 0x09,
    0x55, 0x47, 0x04, 0x5a, 0x8c, 0xc7, 0x58, 0x5f, 0x75, 0xec, 0x2e, 0xf8, 0xad, 0xd5, 0x45, 0xa8,
    0x10, 0x3e, 0xa7, 0xf4, 0x1e, 0xe8, 0x0c, 0xe1, 0xde, 0xb0, 0x36, 0xbc, 0x1d, 0x62, 0xb0, 0xd3,
    0x3f, 0x2f, 0x95, 0x70, 0xc1, 0xc3, 0x1a, 0x25, 0x66, 0xa7, 0x37, 0x0a, 0x87, 0x49, 0xba, 0xbc,
    0x29, 0x27, 0xfd, 0x13, 0xdf, 0x03, 0xf9, 0x94, 0x8b, 0x92, 0x03, 0x6a, 0x70, 0x0d, 0xeb, 0x31,
    0xb5, 0xb3, 0x7d, 0x5d, 0x82, 0x19, 0x78, 0x44, 0xc0, 0x25, 0xa6, 0xff, 0x27, 0x3a, 0x81, 0xf4,
    0x05, 0xab, 0x76, 0xb0, 0x90, 0xe7, 0xbb, 0x24, 0x53, 0xb0, 0x3f, 0xb7, 0x4e, 0x29, 0xe6, 0x87,
    0x07, 0xde, 0x4c, 0x5a, 0x3d, 0x79, 0x99, 0x28, 0x50, 0x4b, 0xf1, 0xd7, 0x7b, 0x07, 0x76, 0xe1,
    0xc3, 0xeb, 0x5d, 0x73, 0x79, 0x48, 0xbb, 0x22, 0x96, 0xdd, 0xe6, 0x0b, 0x8b, 0xf4, 0x14, 0x34,
    0x06, 0xc6, 0xbe, 0x09, 0x49, 0x30, 0x4f, 0x19, 0xba, 0x2d, 0x4c, 0x9b, 0xa3, 0x24, 0x25, 0xfd,
    0xe5, 0xdb, 0xec, 0x99, 0x33, 0x69, 0x27, 0xa4, 0xa3, 0xd6, 0xd4, 0x92, 0xe8, 0xec, 0xa6, 0xa5,
    0x26, 0x15, 0xd7, 0x21, 0x2b, 0x5f, 0x89, 0xe7, 0x70, 0x69, 0xe9, 0xd5, 0x2e, 0x12, 0x39, 0x8f,
    0x8a, 0x99, 0xf6, 0x37, 0xeb, 0xa8, 0x20, 0x5b, 0xa1, 0x58, 0x3c, 0x94, 0x0a, 0xe5, 0x96, 0x4f,
    0x20, 0x55, 0x50, 0x67, 0xe5, 0xa9, 0xe2, 0x70, 0xe9, 0x96, 0x4a, 0x13, 0x00, 0x07, 0xb8, 0x28,
    0xb2, 0x5c, 0xbf, 0x2b, 0x52, 0x48, 0x28, 0xb5, 0x4a, 0xaf, 0x41, 0xef, 0xdf, 0x55, 0xa2, 0x43,
    0x89, 0xc0, 0xcd, 0xbf, 0xd5, 0x90, 0x4d, 0x4d, 0x54, 0x46, 0xd7, 0x64, 0xc7, 0x62, 0x4d, 0xf1,
    0x16, 0x52, 0x7c, 0x89, 0xa8, 0xcc, 0x86, 0x78, 0xca, 0xd2, 0xa2, 0x45, 0xf3, 0x76, 0x48, 0xe5,
    0xbd, 0x76, 0xa3, 0xc5, 0x6c, 0x70, 0xa8, 0xcf, 0x51, 0x5c, 0x49, 0x41, 0xee, 0x6d, 0xef, 0x2c,
    0xac, 0xfc, 0xfa, 0x79, 0xf0, 0x15, 0x1a, 0xe7, 0xe4, 0x47, 0x24, 0x0a, 0xc2, 0x2c, 0x3b, 0x94,
    0x74, 0x4d, 0x3a, 0x5c, 0xa7, 0x73, 0x62, 0xca, 0xe5, 0xac, 0x72, 0x22, 0xdb, 0x4e, 0xb0, 0x55,
    0x49, 0x0c, 0x53, 0x08, 0x8c, 0xbe, 0x35, 0x8f, 0x66, 0xe8, 0x40, 0xaa, 0x7c, 0xd0, 0x17, 0xf5,
    0x91, 0x0e, 0x6f, 0xec, 0x5e, 0xb5, 0x1d, 0x33, 0xf0, 0x79, 0x01, 0x58, 0xec, 0x28, 0x4c, 0x78,
    0x17, 0x25, 0xf2, 0xec, 0xb0, 0xbc, 0x45, 0xad, 0x2a, 0x4e, 0x14, 0x71, 0xc4, 0x67, 0x59, 0xff,
    0xe6, 0x37, 0x22, 0xcb, 0x0d, 0xbc, 0x5b, 0xfc, 0x58, 0xd0, 0x26, 0x76, 0x5e, 0x8a, 0xbc, 0x33,
    0xfb, 0xec, 0x52, 0x01, 0xbf, 0xcb, 0xd3, 0xf3, 0x43, 0x3b, 0x6b, 0x54, 0x8f, 0x78, 0x00, 0xb7,
    0xe6, 0xba, 0xca, 0xbb, 0x63, 0xb6, 0x2b, 0xd6, 0x06, 0x96, 0x20, 0x5d, 0x7b, 0x85, 0xe9, 0x3f,
    0x54, 0xe8, 0x4f, 0x12, 0xce, 0x84, 0x1f, 0x21, 0x62, 0x5d, 0xb2, 0xfa, 0xe2, 0x05, 0x52, 0x71,
    0x47, 0x3e, 0xe1, 0x7f, 0x2f, 0x65, 0x09, 0x6b, 0x28, 0xe4, 0x81, 0x1b, 0xa4, 0x7e, 0xe9, 0x95,
    0x8c, 0x45, 0xd7, 0x06, 0x63, 0x48, 0x73, 0x69, 0xe2, 0x53, 0xab, 0xc7, 0x66, 0xda, 0x1c, 0x35,
    0x95, 0x44, 0xf3, 0x2e, 0x8a, 0x95, 0x4d, 0xde, 0xcf, 0x2e, 0x70, 0x03, 0x70, 0x43, 0x31, 0xf0,
    0x96, 0xf8, 0x8d, 0xb2, 0xfa, 0x3b, 0x76, 0x99, 0x9a, 0x24, 0x22, 0xd9, 0x39, 0xea, 0x12, 0x09,
    0x08, 0x04, 0xca, 0x14, 0x91, 0x78, 0x8e, 0x49, 0x6f, 0x52, 0x7a, 0xb0, 0x36, 0x9d, 0x26, 0x21,
    0xce, 0xb8, 0xea, 0x47, 0xa0, 0x24, 0x70, 0x4e, 0x2f, 0x18, 0xcd, 0xac, 0x5a, 0x2f, 0xe6, 0x0a,
    0xae, 0xcd, 0x17, 0xbb, 0xb4, 0xf4, 0x75, 0xb0, 0x34, 0x0f, 0xa6, 0x9d, 0xfd, 0x7c, 0x76, 0xf2,
    0x5d, 0x33, 0xf2, 0xe4, 0xa5, 0xa5, 0x8e, 0x04, 0xf7, 0x75, 0xb4, 0xfc, 0xa0, 0xca, 0x82, 0xab,
    0xa4, 0x72, 0xf7, 0x79, 0x42, 0x44, 0xc8, 0xa7, 0xb2, 0x0a, 0xf6, 0x8b, 0x81, 0xe9, 0x16, 0x64,
    0x2b, 0x7b, 0xbe, 0xd4, 0x3b, 0x94, 0x60, 0x4f, 0x63, 0x7f, 0x86, 0x73, 0x8d, 0x3c, 0x28, 0x2d,
    0x06, 0x98, 0x73, 0x20, 0x09, 0x6b, 0x46, 0x0f, 0x50, 0x18, 0xa1, 0xb0, 0xb7, 0x51, 0x26, 0x27,
    0x7e, 0x01, 0xea, 0x87, 0xd6, 0xfb, 0x1a, 0xf1, 0xa5, 0x4f, 0x9e, 0xc0, 0x9e, 0xe8, 0x60, 0x31,
    0x16, 0x38, 0x8c, 0xdf, 0xdf, 0x44, 0xb7, 0xee, 0x0d, 0x82, 0xbb, 0x1b, 0x9d, 0x87, 0x1c, 0x99,
    0x43, 0x51, 0xa4, 0xfb, 0x6a, 0x2e, 0xac, 0x25, 0xee, 0x67, 0x86, 0x20, 0x4f, 0x0e, 0x6a, 0xc9,
    0x76, 0x91, 0x49, 0xc1, 0x8d, 0x19, 0x1e, 0x4e, 0xe7, 0xe4, 0xe0, 0x2e, 0x18, 0x77, 0x2a, 0x5b,
    0x9e, 0x50, 0x71, 0x7b, 0xa4, 0x35, 0xf4, 0x0d, 0x2b, 0x8a, 0x8a, 0x69, 0x59, 0x28, 0x63, 0x94,
    0x1a, 0x85, 0x85, 0x17, 0x26, 0x22, 0xd0, 0x6c, 0x91, 0x35, 0xb7, 0x58, 0x31, 0xee, 0x89, 0x94,
    0x0b, 0xdf, 0x5f, 0xb3, 0x19, 0xa3, 0x88, 0x5c, 0x7d, 0x22, 0xb2, 0xea, 0x36, 0x5b, 0xb3, 0x23,
    0x87, 0x3e, 0xb7, 0x63, 0x75, 0x14, 0xc3, 0x0f, 0x5a, 0xaf, 0xab, 0xe3, 0x97, 0xd7, 0xf0, 0x41,
    0x07, 0x22, 0x86, 0xdf, 0x7f, 0x70, 0x85, 0x9e, 0xd0, 0x40, 0x23, 0x5e, 0x93, 0x9a, 0x65, 0xf4,
    0x47, 0x45, 0xbd, 0xdc, 0xd1, 0xcd, 0x3e, 0xc4, 0xf7, 0x15, 0x5e, 0xd7, 0x73, 0xae, 0xbe, 0x93,
    0x60, 0x32, 0x24, 0x51, 0xdb, 0x36, 0xe1, 0x5e, 0x44, 0xe6, 0x1c, 0xda, 0xd6, 0xb4, 0xbd, 0x2f,
    0xb0, 0x12, 0xbc, 0x57, 0xa0, 0xfb, 0x30, 0xe3, 0x64, 0x73, 0x62, 0x11, 0x8d, 0xcb, 0x63, 0x17,
    0xb4, 0xe8, 0x45, 0xa1, 0x8d, 0xe7, 0x81, 0x22, 0x91, 0x04, 0xec, 0x4d, 0x59, 0xe6, 0xf9, 0x6e,
    0x10, 0x96, 0x13, 0x5b, 0x7e, 0xc8, 0x6f, 0x01, 0x8b, 0xcd, 0x33, 0x04, 0x77, 0x82, 0x2b, 0xbc,
    0xae, 0x80, 0x3f, 0x85, 0x47, 0x28, 0x21, 0xe3, 0x4c, 0x51, 0x27, 0xa9, 0xfe, 0xe6, 0x0c, 0xc8,
    0x81, 0x41, 0xff, 0xd9, 0x85, 0x82, 0x30, 0xa1, 0xeb, 0xbb, 0x8d, 0xdf, 0x52, 0x4e, 0xbd, 0x36,
    0x7e, 0x56, 0xc8, 0x2c, 0x9a, 0xed, 0xae, 0xbd, 0x0d, 0xf2, 0xab, 0xf9, 0x95, 0x88, 0xb0, 0x90,
    0x99, 0x6e, 0x2a, 0x01, 0xc2, 0xf7, 0xb8, 0x08, 0xca, 0x8e, 0x35, 0x60, 0xe6, 0x18, 0x73, 0xd8,
    0xe6, 0x77, 0xf1, 0xfb, 0x95, 0x21, 0xa4, 0xcc, 0x56, 0xb2, 0xba, 0x32, 0xbb, 0xca, 0xe6, 0xc7,
    0x3a, 0x3d, 0x3a, 0x18, 0x31, 0xcb, 0x4d, 0x7a, 0x55, 0x84, 0xbb, 0xa2, 0x33, 0x16, 0x36, 0xe7,
    0x41, 0xa7, 0x82, 0xb7, 0xd1, 0x3f, 0x78, 0x2e, 0x23, 0x99, 0xa1, 0x9b, 0xcd, 0xe8, 0x40, 0x34,
    0x21, 0x13, 0x11, 0x28, 0x4b, 0xb0, 0xf0, 0x9d, 0x03, 0x2a, 0x54, 0xca, 0x38, 0x23, 0xfc, 0x48,
    0xdc, 0x5c, 0xfd, 0x33, 0xa6, 0x09, 0xc5, 0x3a, 0x42, 0xd7, 0xce, 0xe5, 0xf4, 0x35, 0xa8, 0x93,
    0x32, 0xf1, 0x8a, 0x1b, 0xd9, 0x94, 0x2b, 0xd0, 0x29, 0x62, 0xea, 0x9a, 0x38, 0xbd, 0xf0, 0xcc,
    0x2e, 0x35, 0xb2, 0x16, 0xb9, 0x25, 0xaf, 0xa6, 0x1d, 0x72, 0x4b, 0x51, 0xec, 0x53, 0x49, 0xba,
    0xe8, 0x40, 0xa9, 0xb1, 0xde, 0xc4, 0x4e, 0x7c, 0x20, 0x5f, 0x4f, 0xd0, 0x7f, 0xb0, 0x79, 0xbc,
    0x8c, 0x6e, 0x35, 0xb1, 0x38, 0x30, 0xd9, 0xa8, 0xb5, 0xbf, 0x55, 0xb8, 0x6c, 0x34, 0xdd, 0x28,
    0xce, 0xf0, 0xe9, 0xd0, 0x4e, 0x06, 0x6c, 0x82, 0x9c, 0xad, 0x7c, 0xc2, 0xf8, 0x88, 0xfc, 0x7d,
    0x03, 0x2c, 0x8e, 0xd6, 0xfb, 0x56, 0x88, 0x41, 0x0e, 0x66, 0xa5, 0x87, 0xde, 0xd2, 0x37, 0xe2,
    0x75, 0x3b, 0x4c, 0x82, 0xb4, 0x64, 0x74, 0x70, 0xf0, 0x5d, 0x97, 0xac, 0x74, 0xe2, 0x97, 0xd5,
    0xf9, 0xac, 0xb9, 0x32, 0x92, 0xa8, 0xf4, 0x12, 0x12, 0xa0, 0x0b, 0x59, 0x81, 0x91, 0x5f, 0x84,
    0xa4, 0x22,
];

const PACKET2_BYTES: &[u8] = &[
    0xc6, 0x00, 0x00, 0x00, 0x01, 0x08, 0xc8, 0xce, 0x86, 0x5d, 0x32, 0xec, 0x58, 0x20, 0x00, 0x00,
    0x44, 0xd0, 0x1b, 0x3f, 0xb0, 0x20, 0xa7, 0x07, 0xdf, 0x29, 0xcc, 0x95, 0xe9, 0x22, 0xa3, 0xc3,
    0x79, 0xa2, 0x60, 0x95, 0x54, 0xc9, 0x5f, 0xef, 0xfe, 0xce, 0x64, 0x21, 0x06, 0xf4, 0xb5, 0xef,
    0x9e, 0xf3, 0xaa, 0x9b, 0x77, 0x61, 0x80, 0x78, 0xe8, 0xb4, 0x13, 0x44, 0xbb, 0x22, 0x2f, 0x01,
    0x88, 0x9b, 0xe1, 0x5d, 0x33, 0xfb, 0xf0, 0x54, 0xcd, 0x86, 0x80, 0xa1, 0xed, 0x83, 0x73, 0x12,
    0x29, 0x59, 0xf8, 0xcc, 0x07, 0x9d, 0x39, 0xa8, 0xf5, 0x4f, 0x85, 0x61, 0x6d, 0x9c, 0x10, 0x7e,
    0x23, 0xf3, 0x87, 0x65, 0xc3, 0xe9, 0x50, 0xfd, 0x0b, 0xa4, 0x59, 0xd6, 0xcd, 0x18, 0x65, 0x33,
    0x50, 0xe7, 0x05, 0xee, 0xd0, 0xdf, 0xa0, 0xa0, 0x1c, 0x16, 0xbc, 0xaf, 0xd0, 0x2e, 0xd6, 0xb5,
    0x7c, 0x5e, 0x87, 0x66, 0x77, 0x57, 0x0b, 0x5a, 0x9b, 0x0b, 0xd8, 0xc2, 0xc0, 0xf5, 0x6b, 0xe2,
    0x37, 0x6e, 0x79, 0xd8, 0x51, 0x60, 0xd9, 0x6a, 0xc4, 0x4c, 0x69, 0xe8, 0xad, 0x63, 0xb5, 0xda,
    0x98, 0x89, 0x4a, 0x91, 0xc0, 0xdf, 0xa6, 0x63, 0x0c, 0x4f, 0x62, 0xef, 0x88, 0xb5, 0x3e, 0xd8,
    0x77, 0x24, 0x5a, 0x61, 0x03, 0x9c, 0x18, 0x25, 0xef, 0xf2, 0xa9, 0x81, 0x64, 0x56, 0x4f, 0x7f,
    0x44, 0x3d, 0xd0, 0xc3, 0x03, 0x1a, 0x5f, 0xd6, 0x6c, 0x8f, 0x3d, 0x19, 0xaa, 0x4a, 0x8b, 0xd5,
    0x9c, 0xe5, 0x67, 0xa4, 0xa9, 0xd3, 0xfe, 0x38, 0x5c, 0x69, 0xdd, 0xd2, 0x23, 0xc9, 0xd6, 0x07,
    0x61, 0xbb, 0x10, 0x00, 0xeb, 0x76, 0x4e, 0xca, 0xed, 0xce, 0x10, 0x12, 0x40, 0x93, 0xe5, 0xb0,
    0xad, 0xed, 0xad, 0x19, 0x04, 0x90, 0xbd, 0x07, 0x19, 0xa1, 0xcf, 0x56, 0x4f, 0xa3, 0x56, 0x19,
    0xb0, 0x05, 0x2d, 0x1a, 0xe2, 0xaf, 0x8f, 0x5a, 0xe8, 0xff, 0x7c, 0xe1, 0x37, 0x73, 0x61, 0xb6,
    0xf1, 0xb7, 0x4f, 0xf8, 0x04, 0x0c, 0x34, 0xa2, 0x4f, 0x32, 0xdc, 0x7a, 0xce, 0x57, 0xb0, 0xa7,
    0x0f, 0x46, 0x75, 0x2f, 0x68, 0xba, 0x07, 0xff, 0xaa, 0x36, 0xa7, 0x55, 0x00, 0x47, 0x85, 0x7f,
    0x77, 0x92, 0x6d, 0xf0, 0xcf, 0x80, 0x7c, 0x29, 0xfe, 0xa3, 0x26, 0xd0, 0x34, 0x16, 0xe4, 0xd8,
    0xc6, 0xd3, 0x62, 0x42, 0x0c, 0x4b, 0x1d, 0xf2, 0x90, 0x61, 0x58, 0x18, 0x86, 0xe3, 0xd6, 0xdd,
    0xf2, 0x42, 0xa2, 0xe0, 0xb4, 0x31, 0x9a, 0x2c, 0x43, 0xf3, 0x76, 0xd8, 0x5d, 0x51, 0x3f, 0xfc,
    0x00, 0xa7, 0xd9, 0x9d, 0xc6, 0xe0, 0x28, 0x2a, 0x14, 0x4d, 0x20, 0x72, 0x2f, 0xfb, 0xe3, 0x0e,
    0xd5, 0xe7, 0x11, 0xaf, 0xa7, 0x91, 0xee, 0xc6, 0xe9, 0xef, 0x0e, 0x03, 0xb4, 0x25, 0x1a, 0x68,
    0xf0, 0x71, 0x1f, 0xbc, 0xe5, 0x3e, 0x7d, 0x11, 0x6b, 0xb0, 0xaa, 0x79, 0xcb, 0xe3, 0x7e, 0x87,
    0xa5, 0x04, 0x27, 0xfc, 0x22, 0x1d, 0xa0, 0xd2, 0xfc, 0x13, 0xc2, 0xa2, 0xc0, 0xee, 0x70, 0x16,
    0xdb, 0x9f, 0xd3, 0xbc, 0x85, 0x20, 0x7a, 0x44, 0x8c, 0xd1, 0xe4, 0x8d, 0xe4, 0x97, 0x05, 0xa0,
    0xae, 0xbb, 0x03, 0x7b, 0xe6, 0x09, 0x64, 0x49, 0x4b, 0xf6, 0x90, 0x78, 0xd5, 0x22, 0x76, 0xb1,
    0xe5, 0x18, 0x9a, 0x6c, 0xaf, 0x98, 0xfd, 0xe3, 0xa5, 0x4a, 0xd1, 0x9b, 0x13, 0xaf, 0xf3, 0xa0,
    0xbc, 0x7c, 0x9b, 0x11, 0x8c, 0x04, 0x5b, 0x34, 0xa5, 0x27, 0x62, 0x63, 0xdd, 0x80, 0xc1, 0xa9,
    0xff, 0x50, 0xef, 0x73, 0xda, 0x59, 0x81, 0x31, 0x7a, 0x8d, 0x88, 0x77, 0x4d, 0x24, 0x9b, 0xfc,
    0x92, 0xdc, 0xa5, 0x2e, 0x25, 0x41, 0x5f, 0xf0, 0xf9, 0x5b, 0x5f, 0xa5, 0x20, 0xed, 0x00, 0x88,
    0xc8, 0xb4, 0x77, 0x1c, 0xf5, 0xf5, 0x3d, 0xd6, 0xcf, 0x25, 0x4f, 0xbb, 0x3a, 0xc0, 0xb2, 0x25,
    0x1d, 0x52, 0xe4, 0x63, 0xfd, 0xe5, 0xc1, 0x1a, 0xcc, 0xa8, 0xb5, 0x9b, 0x4c, 0x4b, 0xb3, 0x73,
    0xa5, 0x12, 0xb4, 0x73, 0x31, 0x07, 0xcc, 0x77, 0xcc, 0xc6, 0xae, 0x16, 0xcb, 0xd8, 0x5a, 0x2f,
    0xa8, 0x98, 0x33, 0x57, 0x90, 0xc9, 0x85, 0x6c, 0xca, 0x83, 0x3f, 0xce, 0x2e, 0x0a, 0x08, 0xa9,
    0x8b, 0x29, 0x28, 0x79, 0x6b, 0x3f, 0x05, 0x5b, 0x3c, 0xfe, 0xec, 0x1a, 0xad, 0x10, 0x75, 0x67,
    0x48, 0xf5, 0xa8, 0xe8, 0x60, 0xe3, 0xb8, 0xa7, 0x71, 0x63, 0x1e, 0x14, 0x13, 0x2d, 0xcc, 0x88,
    0xe1, 0xdf, 0x97, 0xf2, 0xca, 0x30, 0xfc, 0x22, 0xe8, 0xfa, 0x75, 0x13, 0x50, 0x29, 0x4f, 0xc2,
    0x33, 0xbe, 0xf9, 0x27, 0xe4, 0x29, 0xdc, 0xab, 0x5c, 0xde, 0x76, 0x41, 0x35, 0x06, 0xe5, 0x91,
    0x9e, 0x6d, 0x1e, 0xbc, 0x43, 0xa4, 0x37, 0x05, 0x8a, 0x12, 0xd9, 0xe8, 0x36, 0xee, 0x3d, 0xd5,
    0x20, 0x03, 0x71, 0x10, 0x57, 0xae, 0xc2, 0xf2, 0xb8, 0xae, 0x8b, 0x25, 0xe8, 0x4e, 0xbc, 0xef,
    0xbc, 0xa3, 0x47, 0xe5, 0x55, 0x53, 0x9b, 0x86, 0xf0, 0xc1, 0x3b, 0x81, 0x15, 0x03, 0x73, 0x35,
    0xbb, 0xf9, 0xf6, 0xf7, 0xa6, 0xee, 0x76, 0xa6, 0x06, 0x54, 0x7e, 0x9a, 0x1b, 0xdb, 0x39, 0x08,
    0xc4, 0xd9, 0xe6, 0xd7, 0xf1, 0x62, 0xd8, 0x21, 0x3e, 0xe1, 0x3e, 0x2e, 0x31, 0xe7, 0x6d, 0xe1,
    0x41, 0xa7, 0xef, 0x82, 0xc9, 0xd3, 0xf4, 0x7f, 0x7e, 0x70, 0x98, 0xf0, 0xb2, 0x1a, 0x1f, 0x89,
    0x26, 0xa7, 0x7c, 0x09, 0x78, 0x72, 0x7e, 0x46, 0xcd, 0x37, 0x53, 0xed, 0x41, 0x4a, 0x97, 0x7e,
    0xea, 0xfd, 0x1d, 0x0e, 0xf5, 0xc8, 0x73, 0xf0, 0x1c, 0x1c, 0x12, 0x41, 0xe1, 0xbd, 0x0f, 0xe8,
    0x73, 0x1f, 0x0a, 0x11, 0x51, 0x9c, 0x1b, 0xee, 0x31, 0xa4, 0xd4, 0x43, 0xb8, 0x6d, 0x25, 0x12,
    0xf6, 0x51, 0x55, 0xfd, 0xb4, 0xc3, 0x55, 0x9a, 0xc2, 0x49, 0xfb, 0x0e, 0x57, 0x23, 0x92, 0x6c,
    0xbe, 0xfa, 0x0a, 0xcc, 0xb4, 0x53, 0x4a, 0x40, 0x77, 0xdd, 0x92, 0x0e, 0xdf, 0xbd, 0x45, 0x5e,
    0xc1, 0x27, 0x9c, 0xf3, 0xe4, 0xd3, 0x06, 0x49, 0x4c, 0xea, 0x4a, 0xc8, 0x5a, 0x29, 0xd4, 0xb8,
    0xa5, 0x00, 0xc0, 0x94, 0x46, 0xd1, 0x1c, 0x88, 0x4b, 0xef, 0x87, 0xba, 0xdf, 0x98, 0xad, 0x66,
    0x9e, 0x35, 0x74, 0x0b, 0xc9, 0xc5, 0xe3, 0x30, 0x72, 0xf5, 0xbe, 0xfb, 0x2e, 0xf7, 0x3b, 0xdb,
    0x95, 0x56, 0xe8, 0x57, 0x60, 0x6b, 0x41, 0x05, 0x72, 0x80, 0x86, 0xe2, 0x6b, 0x81, 0xb3, 0x5b,
    0xbd, 0x7e, 0x90, 0x1e, 0xf5, 0x5a, 0x3c, 0x57, 0xc6, 0xd6, 0x63, 0x3b, 0x68, 0xb9, 0xdf, 0x58,
    0x5c, 0xf4, 0x0e, 0xc3, 0x34, 0xdf, 0xd3, 0x69, 0xef, 0xbb, 0x1d, 0x15, 0x57, 0x54, 0x6f, 0xb7,
    0xf6, 0xb2, 0x1e, 0x3c, 0x30, 0x17, 0x4c, 0xc6, 0xd2, 0x79, 0x54, 0x63, 0x86, 0xc3, 0xa1, 0x9b,
    0x15, 0xd7, 0xf6, 0x87, 0x28, 0x06, 0x4c, 0x88, 0x99, 0xd7, 0xdb, 0x9d, 0x72, 0xdb, 0xbc, 0xbb,
    0xc7, 0xb6, 0x18, 0x45, 0xe8, 0xf8, 0x18, 0x7a, 0x12, 0xf3, 0x14, 0x74, 0x39, 0xcf, 0x71, 0xc5,
    0xc1, 0x9b, 0x73, 0x1c, 0x94, 0x0d, 0x94, 0x43, 0x84, 0x43, 0x91, 0x1a, 0x2d, 0xd3, 0xdd, 0x3e,
    0xff, 0xfd, 0x5d, 0x60, 0x53, 0x75, 0xbc, 0xb2, 0x84, 0x9a, 0x2e, 0xec, 0xea, 0xe5, 0x90, 0xb0,
    0xb2, 0xaf, 0xf2, 0x57, 0x72, 0x7e, 0x78, 0xe4, 0x00, 0x89, 0xa8, 0x5f, 0xd2, 0xe6, 0x62, 0x31,
    0xee, 0xd8, 0x91, 0xec, 0x07, 0x8b, 0x6f, 0xcf, 0xa7, 0xdd, 0xf3, 0xca, 0x7c, 0x37, 0xf4, 0xee,
    0xbf, 0xa0, 0x99, 0x22, 0x5b, 0x21, 0x55, 0x0d, 0xc3, 0x09, 0xf5, 0x98, 0x02, 0x38, 0x94, 0x63,
    0x23, 0x0f, 0xfb, 0x13, 0x06, 0x6e, 0x91, 0xdf, 0x90, 0x26, 0xbb, 0xd6, 0x92, 0x6d, 0x8a, 0x92,
    0xb0, 0xa4, 0x24, 0x88, 0x0d, 0x42, 0xa4, 0x40, 0xf1, 0x49, 0x87, 0x73, 0xcb, 0x4e, 0x73, 0x95,
    0xa3, 0x9f, 0x70, 0x02, 0xa3, 0x64, 0x9d, 0xa7, 0x7a, 0x9a, 0x73, 0x60, 0x5f, 0x7e, 0x7a, 0x2a,
    0x20, 0xdd, 0x3f, 0x65, 0xcc, 0xc4, 0x2f, 0xfa, 0xd5, 0x78, 0x74, 0xff, 0x44, 0xb6, 0xe0, 0xe1,
    0x36, 0x8e, 0x37, 0xe9, 0x53, 0x14, 0x6d, 0x5b, 0xae, 0x74, 0x6b, 0x88, 0x9b, 0x34, 0xdd, 0x9e,
    0xa6, 0x83, 0x5a, 0x40, 0x73, 0x2b, 0x6b, 0x1c, 0xeb, 0x10, 0x78, 0x24, 0xbb, 0x01, 0x54, 0x9b,
    0xa5, 0xfd, 0x22, 0xf3, 0x08, 0x5f, 0x8c, 0x8b, 0xd5, 0x50, 0x62, 0xda, 0x6e, 0x06, 0x03, 0x4f,
    0xe2, 0x79, 0x64, 0xeb, 0x69, 0x34, 0x18, 0x4f, 0xba, 0xe9, 0xce, 0x23, 0x95, 0x2d, 0x6b, 0xaf,
    0xa0, 0xa1, 0xf1, 0x95, 0xe3, 0x71, 0x15, 0x21, 0x2e, 0x78, 0x6a, 0x6d, 0x20, 0xfd, 0x26, 0x89,
    0x07, 0xfa, 0x34, 0xe2, 0xce, 0x3d, 0x24, 0x9f, 0xa1, 0x4c, 0xb6, 0xa8, 0xeb, 0x91, 0x13, 0xe1,
    0xcc, 0xf1, 0xa1, 0x87, 0xe6, 0x1e, 0xfc, 0x42, 0x6b, 0x23, 0x05, 0x60, 0x18, 0x1a, 0x09, 0x52,
    0x52, 0x35, 0x03, 0xb0, 0x3c, 0xe7, 0xd4, 0x04, 0x0d, 0xe3, 0x54, 0x5c, 0x38, 0x5b, 0x54, 0x19,
    0x1c, 0x60, 0x8c, 0xb5, 0xfa, 0xa0, 0xf0, 0x54, 0x3d, 0xae, 0x7e, 0xb6, 0x59, 0xc8, 0x69, 0xa0,
    0xf7, 0x0f,
];

#[test]
fn sni() {
    let mut handshake_coalescer = HandshakeCoalescer::new(1 << 16);

    let packet1 = InitialPacket::parse_client(PACKET1_BYTES).unwrap();
    assert_eq!(packet1.packet_number(), 1);
    packet1.consume_frames(&mut handshake_coalescer).unwrap();
    assert!(!handshake_coalescer.finished());

    let packet2 = InitialPacket::parse_client(PACKET2_BYTES).unwrap();
    assert_eq!(packet2.packet_number(), 2);
    packet2.consume_frames(&mut handshake_coalescer).unwrap();
    assert!(handshake_coalescer.finished());

    let client_hello = handshake_coalescer.parse_client_hello().unwrap().unwrap();
    let sni_bytes = client_hello
        .get_ext(ExtensionType::ServerName)
        .unwrap()
        .unwrap();
    let sni = TlsServerName::from_extension_value(sni_bytes).unwrap();
    assert_eq!(sni.as_ref(), "accounts.google.com");
}