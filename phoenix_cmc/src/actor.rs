// -*- Rust -*-
//
// $Id$
//
// Phoenix CMC library: actor module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_variables)]
#![allow(dead_code)]

use async_backtrace::{frame, framed};
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::warn;
use trait_variant;

#[trait_variant::make(Actor: Send)]
pub trait LocalActor: Debug + Clone + Sized {
    type Error: Error;
}

#[trait_variant::make(ActorInner: Send)]
pub trait LocalActorInner: Debug + Send + Sized {
    type Error: Error;

    async fn run(self) -> Result<(), Self::Error>;
}
