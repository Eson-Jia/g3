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

use std::io;

use thiserror::Error;

use g3_types::net::ConnectError;

#[derive(Debug, Error)]
pub(crate) enum StreamConnectError {
    #[error("upstream not resolved")]
    UpstreamNotResolved,
    #[error("setup socket failed: {0:?}")]
    SetupSocketFailed(io::Error),
    #[error("connect failed: {0}")]
    ConnectFailed(#[from] ConnectError),
}
