// -*- Rust -*-
//
// Phoenix CMC library: file module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FileError {
    IoError { path: PathBuf, source: IoError },
}

impl FileError {
    fn from_io_error(path: PathBuf, source: IoError) -> Self {
        Self::IoError { path, source }
    }
}

impl Error for FileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError { source, .. } => source.source(),
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError { path, source } => {
                write!(f, "{path}: I/O error: {source}", path = path.display())
            }
        }
    }
}
