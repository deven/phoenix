// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: error module
//
// Copyright 2021-2023 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::session::SessionMessage;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tracing::{trace, warn};

#[derive(Debug)]
pub enum PhoenixError {
    FileIoError {
        path: PathBuf,
        source: std::io::Error,
    },
    SessionTxError(mpsc::error::SendError<SessionMessage>),
    SessionRxError(oneshot::error::RecvError),
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PhoenixError::SessionTxError(err) => Some(err),
            PhoenixError::SessionRxError(err) => Some(err),
            PhoenixError::FileIoError { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhoenixError::SessionTxError(err) => write!(f, "Session Tx error: {}", err),
            PhoenixError::SessionRxError(err) => write!(f, "Session Rx error: {}", err),
            PhoenixError::FileIoError { path, source } => {
                write!(f, "File I/O error for path {}: {}", path.display(), source)
            }
        }
    }
}

impl From<mpsc::error::SendError<SessionMessage>> for PhoenixError {
    fn from(error: mpsc::error::SendError<SessionMessage>) -> Self {
        PhoenixError::SessionTxError(error)
    }
}

impl From<oneshot::error::RecvError> for PhoenixError {
    fn from(error: oneshot::error::RecvError) -> Self {
        PhoenixError::SessionRxError(error)
    }
}
