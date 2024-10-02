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

use std::sync::Arc;

use g3_io_ext::{LimitedUdpRecv, LimitedUdpSend};

use super::{ProxyFloatEscaper, ProxyFloatSocks5sPeer};
use crate::escape::proxy_socks5::udp_relay::{
    ProxySocks5UdpRelayRemoteRecv, ProxySocks5UdpRelayRemoteSend,
};
use crate::module::tcp_connect::TcpConnectTaskNotes;
use crate::module::udp_relay::{
    ArcUdpRelayTaskRemoteStats, UdpRelayRemoteWrapperStats, UdpRelaySetupError,
    UdpRelaySetupResult, UdpRelayTaskNotes,
};
use crate::serve::ServerTaskNotes;

impl ProxyFloatSocks5sPeer {
    pub(super) async fn udp_setup_relay(
        &self,
        escaper: &ProxyFloatEscaper,
        udp_notes: &UdpRelayTaskNotes,
        task_notes: &ServerTaskNotes,
        task_stats: ArcUdpRelayTaskRemoteStats,
    ) -> UdpRelaySetupResult {
        let mut tcp_notes = TcpConnectTaskNotes::empty();
        let (ctl_stream, udp_socket, udp_local_addr, udp_peer_addr) = self
            .timed_socks5_udp_associate(escaper, udp_notes.buf_conf, &mut tcp_notes, task_notes)
            .await
            .map_err(UdpRelaySetupError::SetupSocketFailed)?;

        let mut wrapper_stats = UdpRelayRemoteWrapperStats::new(&escaper.stats, task_stats);
        wrapper_stats.push_user_io_stats(escaper.fetch_user_upstream_io_stats(task_notes));
        let wrapper_stats = Arc::new(wrapper_stats);

        let (recv, send) = g3_io_ext::split_udp(udp_socket);
        let recv = LimitedUdpRecv::local_limited(
            recv,
            self.udp_sock_speed_limit.shift_millis,
            self.udp_sock_speed_limit.max_south_packets,
            self.udp_sock_speed_limit.max_south_bytes,
            wrapper_stats.clone(),
        );
        let send = LimitedUdpSend::local_limited(
            send,
            self.udp_sock_speed_limit.shift_millis,
            self.udp_sock_speed_limit.max_north_packets,
            self.udp_sock_speed_limit.max_north_bytes,
            wrapper_stats,
        );

        let recv = ProxySocks5UdpRelayRemoteRecv::new(
            recv,
            udp_local_addr,
            udp_peer_addr,
            ctl_stream,
            self.end_on_control_closed,
        );
        let send = ProxySocks5UdpRelayRemoteSend::new(send, udp_local_addr, udp_peer_addr);

        Ok((
            Box::new(recv),
            Box::new(send),
            escaper.escape_logger.clone(),
        ))
    }
}