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

use std::time::Duration;
use std::{fmt, str::FromStr};

mod size_limit;

use g3_types::acl::ActionContract;
pub use size_limit::ProtocolInspectionSizeLimit;

mod http;
pub use http::{H1InterceptionConfig, H2InterceptionConfig};

mod smtp;
pub use smtp::SmtpInterceptionConfig;

mod imap;
pub use imap::ImapInterceptionConfig;

pub type ProtocolInspectPolicy = g3_types::acl_set::AclDstHostRuleSet<ProtocolInspectAction>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub enum ProtocolInspectAction {
    Intercept,
    #[cfg(feature = "quic")]
    Detour,
    Bypass,
    Block,
}

impl ProtocolInspectAction {
    #[inline]
    fn as_str(&self) -> &'static str {
        self.serialize()
    }
}

impl fmt::Display for ProtocolInspectAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(ProtocolInspectAction::as_str(self))
    }
}

impl FromStr for ProtocolInspectAction {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s).map_err(|_| ())
    }
}

impl g3_types::acl::ActionContract for ProtocolInspectAction {
    fn default_forbid() -> Self {
        Self::Block
    }

    fn default_permit() -> Self {
        Self::Intercept
    }

    fn restrict(&self, other: &ProtocolInspectAction) -> ProtocolInspectAction {
        if other > self {
            *other
        } else {
            *self
        }
    }

    fn strict_than(&self, other: &ProtocolInspectAction) -> bool {
        self.gt(other)
    }

    fn forbid_early(&self) -> bool {
        match self {
            Self::Block => true,
            Self::Intercept | Self::Bypass => false,
            #[cfg(feature = "quic")]
            Self::Detour => false,
        }
    }

    fn serialize(&self) -> &'static str {
        match self {
            Self::Intercept => "intercept",
            Self::Block => "block",
            Self::Bypass => "bypass",
            #[cfg(feature = "quic")]
            Self::Detour => "detour",
        }
    }

    fn deserialize(s: &str) -> Result<Self, &str> {
        match s.to_lowercase().as_str() {
            "intercept" => Ok(ProtocolInspectAction::Intercept),
            #[cfg(feature = "quic")]
            "detour" => Ok(ProtocolInspectAction::Detour),
            "bypass" => Ok(ProtocolInspectAction::Bypass),
            "block" => Ok(ProtocolInspectAction::Block),
            _ => Err(s),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolInspectionConfig {
    inspect_max_depth: usize,
    data0_buffer_size: usize,
    data0_wait_timeout: Duration,
    data0_read_timeout: Duration,
    data0_size_limit: ProtocolInspectionSizeLimit,
}

impl Default for ProtocolInspectionConfig {
    fn default() -> Self {
        ProtocolInspectionConfig {
            inspect_max_depth: 4,
            data0_buffer_size: 4096,
            data0_wait_timeout: Duration::from_secs(60),
            data0_read_timeout: Duration::from_secs(4),
            data0_size_limit: Default::default(),
        }
    }
}

impl ProtocolInspectionConfig {
    pub fn set_max_depth(&mut self, depth: usize) {
        self.inspect_max_depth = depth;
    }

    #[inline]
    pub fn max_depth(&self) -> usize {
        self.inspect_max_depth
    }

    pub fn set_data0_buffer_size(&mut self, size: usize) {
        self.data0_buffer_size = size;
    }

    #[inline]
    pub fn data0_buffer_size(&self) -> usize {
        self.data0_buffer_size
    }

    #[inline]
    pub fn set_data0_wait_timeout(&mut self, value: Duration) {
        self.data0_wait_timeout = value;
    }

    #[inline]
    pub fn data0_wait_timeout(&self) -> Duration {
        self.data0_wait_timeout
    }

    #[inline]
    pub fn set_data0_read_timeout(&mut self, value: Duration) {
        self.data0_read_timeout = value;
    }

    #[inline]
    pub fn data0_read_timeout(&self) -> Duration {
        self.data0_read_timeout
    }

    #[inline]
    pub fn size_limit(&self) -> &ProtocolInspectionSizeLimit {
        &self.data0_size_limit
    }

    #[inline]
    pub fn size_limit_mut(&mut self) -> &mut ProtocolInspectionSizeLimit {
        &mut self.data0_size_limit
    }
}
