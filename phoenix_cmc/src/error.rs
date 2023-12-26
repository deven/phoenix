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

use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use tracing::{trace, warn};

#[derive(Debug)]
pub enum PhoenixError {
    //RequestError(reqwest::Error),
    FileIoError {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            //PhoenixError::RequestError(err) => Some(err),
            PhoenixError::FileIoError { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //PhoenixError::RequestError(err) => write!(f, "Request error: {}", err),
            PhoenixError::FileIoError { path, source } => {
                write!(f, "File I/O error for path {}: {}", path.display(), source)
            }
        }
    }
}

//impl From<reqwest::Error> for PhoenixError {
//    fn from(error: reqwest::Error) -> Self {
//        PhoenixError::RequestError(error)
//    }
//}
