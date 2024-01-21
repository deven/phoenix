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
use crate::file::FileError;
use crate::server::ServerError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PhoenixError {
    ClientError(ClientError),
    FileError(FileError),
    ServerError(ServerError),
}

impl Error for PhoenixError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ClientError(err) => err.source(),
            Self::FileError(err) => err.source(),
            Self::ServerError(err) => err.source(),
        }
    }
}

impl fmt::Display for PhoenixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientError(err) => err.fmt(f),
            Self::FileError(err) => err.fmt(f),
            Self::ServerError(err) => err.fmt(f),
        }
    }
}

impl From<ClientError> for PhoenixError {
    fn from(err: ClientError) -> Self {
        Self::ClientError(err)
    }
}

impl From<FileError> for PhoenixError {
    fn from(err: FileError) -> Self {
        Self::FileError(err)
    }
}

impl From<ServerError> for PhoenixError {
    fn from(err: ServerError) -> Self {
        Self::ServerError(err)
    }
}
