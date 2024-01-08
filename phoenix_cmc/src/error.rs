// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: error module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use crate::client::ClientError;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum PhoenixError {
    FileIoError {
        path: PathBuf,
        source: std::io::Error,
    },
    ClientError(ClientError),
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ClientError(err) => err.source(),
            Self::FileIoError { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientError(err) => err.fmt(f),
            Self::FileIoError { path, source } => {
                write!(f, "File I/O error for path {}: {}", path.display(), source)
            }
        }
    }
}

impl From<ClientError> for PhoenixError {
    fn from(err: ClientError) -> Self {
        Self::ClientError(err)
    }
}
