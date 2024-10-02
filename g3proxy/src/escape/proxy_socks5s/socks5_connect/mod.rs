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

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use anyhow::anyhow;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UdpSocket;

use g3_daemon::stat::remote::{
    ArcTcpConnectionTaskRemoteStats, TcpConnectionTaskRemoteStatsWrapper,
};
use g3_io_ext::{AsyncStream, LimitedReader, LimitedWriter};
use g3_openssl::{SslConnector, SslStream};
use g3_socket::BindAddr;
use g3_socks::v5;
use g3_types::net::{Host, OpensslClientConfig, SocketBufferConfig};

use super::ProxySocks5sEscaper;
use crate::log::escape::tls_handshake::{EscapeLogForTlsHandshake, TlsApplication};
use crate::module::tcp_connect::{TcpConnectError, TcpConnectResult, TcpConnectTaskNotes};
use crate::serve::ServerTaskNotes;

impl ProxySocks5sEscaper {
    async fn socks5_connect_tcp_connect_to<'a>(
        &'a self,
        tcp_notes: &'a mut TcpConnectTaskNotes,
        task_notes: &'a ServerTaskNotes,
    ) -> Result<SslStream<impl AsyncRead + AsyncWrite>, TcpConnectError> {
        let mut stream = self.tls_handshake_to_remote(tcp_notes, task_notes).await?;
        let outgoing_addr =
            v5::client::socks5_connect_to(&mut stream, &self.config.auth_info, &tcp_notes.upstream)
                .await?;
        tcp_notes.chained.outgoing_addr = Some(outgoing_addr);
        // we can not determine the real upstream addr that the proxy choose to connect to

        Ok(stream)
    }

    pub(super) async fn timed_socks5_connect_tcp_connect_to<'a>(
        &'a self,
        tcp_notes: &'a mut TcpConnectTaskNotes,
        task_notes: &'a ServerTaskNotes,
    ) -> Result<SslStream<impl AsyncRead + AsyncWrite>, TcpConnectError> {
        tokio::time::timeout(
            self.config.peer_negotiation_timeout,
            self.socks5_connect_tcp_connect_to(tcp_notes, task_notes),
        )
        .await
        .map_err(|_| TcpConnectError::NegotiationPeerTimeout)?
    }

    /// setup udp associate with remote proxy
    /// return (socket, listen_addr, peer_addr)
    async fn socks5_udp_associate(
        &self,
        buf_conf: SocketBufferConfig,
        tcp_notes: &mut TcpConnectTaskNotes,
        task_notes: &ServerTaskNotes,
    ) -> Result<
        (
            SslStream<impl AsyncRead + AsyncWrite>,
            UdpSocket,
            SocketAddr,
            SocketAddr,
        ),
        io::Error,
    > {
        let mut ctl_stream = self
            .tls_handshake_to_remote(tcp_notes, task_notes)
            .await
            .map_err(io::Error::other)?;
        let local_tcp_addr = tcp_notes
            .local
            .ok_or_else(|| io::Error::other("no local tcp address"))?;
        let peer_tcp_addr = tcp_notes
            .next
            .ok_or_else(|| io::Error::other("no peer tcp address"))?;

        // bind early and send listen_addr if configured ?
        let send_udp_ip = match local_tcp_addr.ip() {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        };
        let send_udp_addr = SocketAddr::new(send_udp_ip, 0);

        let peer_udp_addr = v5::client::socks5_udp_associate(
            &mut ctl_stream,
            &self.config.auth_info,
            send_udp_addr,
        )
        .await
        .map_err(io::Error::other)?;
        let peer_udp_addr = self
            .config
            .transmute_udp_peer_addr(peer_udp_addr, peer_tcp_addr.ip());
        let socket = g3_socket::udp::new_std_socket_to(
            peer_udp_addr,
            &BindAddr::Ip(local_tcp_addr.ip()),
            buf_conf,
            self.config.udp_misc_opts,
        )?;
        let socket = UdpSocket::from_std(socket)?;
        socket.connect(peer_udp_addr).await?;
        let listen_addr = socket.local_addr()?;

        Ok((ctl_stream, socket, listen_addr, peer_udp_addr))
    }

    pub(super) async fn timed_socks5_udp_associate(
        &self,
        buf_conf: SocketBufferConfig,
        tcp_notes: &mut TcpConnectTaskNotes,
        task_notes: &ServerTaskNotes,
    ) -> Result<
        (
            SslStream<impl AsyncRead + AsyncWrite>,
            UdpSocket,
            SocketAddr,
            SocketAddr,
        ),
        io::Error,
    > {
        tokio::time::timeout(
            self.config.peer_negotiation_timeout,
            self.socks5_udp_associate(buf_conf, tcp_notes, task_notes),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "peer negotiation timeout"))?
    }

    pub(super) async fn socks5_new_tcp_connection<'a>(
        &'a self,
        tcp_notes: &'a mut TcpConnectTaskNotes,
        task_notes: &'a ServerTaskNotes,
        task_stats: ArcTcpConnectionTaskRemoteStats,
    ) -> TcpConnectResult {
        let ups_s = self
            .timed_socks5_connect_tcp_connect_to(tcp_notes, task_notes)
            .await?;

        // add task and user stats
        let mut wrapper_stats = TcpConnectionTaskRemoteStatsWrapper::new(task_stats);
        wrapper_stats.push_other_stats(self.fetch_user_upstream_io_stats(task_notes));
        let wrapper_stats = Arc::new(wrapper_stats);

        let (r, w) = ups_s.into_split();
        let r = LimitedReader::new(r, wrapper_stats.clone());
        let w = LimitedWriter::new(w, wrapper_stats);

        Ok((Box::new(r), Box::new(w)))
    }

    pub(super) async fn socks5_connect_tls_connect_to<'a>(
        &'a self,
        tcp_notes: &'a mut TcpConnectTaskNotes,
        task_notes: &'a ServerTaskNotes,
        tls_config: &'a OpensslClientConfig,
        tls_name: &'a Host,
        tls_application: TlsApplication,
    ) -> Result<SslStream<impl AsyncRead + AsyncWrite>, TcpConnectError> {
        let ups_s = self
            .timed_socks5_connect_tcp_connect_to(tcp_notes, task_notes)
            .await?;

        let ssl = tls_config
            .build_ssl(tls_name, tcp_notes.upstream.port())
            .map_err(TcpConnectError::InternalTlsClientError)?;
        let connector = SslConnector::new(ssl, ups_s)
            .map_err(|e| TcpConnectError::InternalTlsClientError(anyhow::Error::new(e)))?;

        match tokio::time::timeout(tls_config.handshake_timeout, connector.connect()).await {
            Ok(Ok(stream)) => Ok(stream),
            Ok(Err(e)) => {
                let e = anyhow::Error::new(e);
                EscapeLogForTlsHandshake {
                    tcp_notes,
                    task_id: &task_notes.id,
                    tls_name,
                    tls_peer: &tcp_notes.upstream,
                    tls_application,
                }
                .log(&self.escape_logger, &e);
                Err(TcpConnectError::UpstreamTlsHandshakeFailed(e))
            }
            Err(_) => {
                let e = anyhow!("upstream tls handshake timed out");
                EscapeLogForTlsHandshake {
                    tcp_notes,
                    task_id: &task_notes.id,
                    tls_name,
                    tls_peer: &tcp_notes.upstream,
                    tls_application,
                }
                .log(&self.escape_logger, &e);
                Err(TcpConnectError::UpstreamTlsHandshakeTimeout)
            }
        }
    }

    pub(super) async fn socks5_new_tls_connection<'a>(
        &'a self,
        tcp_notes: &'a mut TcpConnectTaskNotes,
        task_notes: &'a ServerTaskNotes,
        task_stats: ArcTcpConnectionTaskRemoteStats,
        tls_config: &'a OpensslClientConfig,
        tls_name: &'a Host,
    ) -> TcpConnectResult {
        let tls_stream = self
            .socks5_connect_tls_connect_to(
                tcp_notes,
                task_notes,
                tls_config,
                tls_name,
                TlsApplication::TcpStream,
            )
            .await?;

        let (ups_r, ups_w) = tls_stream.into_split();

        // add task and user stats
        let mut wrapper_stats = TcpConnectionTaskRemoteStatsWrapper::new(task_stats);
        wrapper_stats.push_other_stats(self.fetch_user_upstream_io_stats(task_notes));
        let wrapper_stats = Arc::new(wrapper_stats);

        let ups_r = LimitedReader::new(ups_r, wrapper_stats.clone());
        let ups_w = LimitedWriter::new(ups_w, wrapper_stats);

        Ok((Box::new(ups_r), Box::new(ups_w)))
    }
}