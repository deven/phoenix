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

use crate::client::session::SessionError;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum PhoenixError {
    FileIoError {
        path: PathBuf,
        source: std::io::Error,
    },
    SessionError(SessionError),
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SessionError(err) => err.source(),
            Self::FileIoError { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionError(err) => err.fmt(f),
            Self::FileIoError { path, source } => {
                write!(f, "File I/O error for path {}: {}", path.display(), source)
            }
        }
    }
}

impl From<SessionError> for PhoenixError {
    fn from(err: SessionError) -> Self {
        Self::SessionError(err)
    }
}
