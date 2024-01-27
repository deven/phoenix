// -*- Rust -*-
//
// Phoenix CMC library: actor module
//
// Copyright 2021-2024 Deven T. Corzine <deven@ties.org>
//
// SPDX-License-Identifier: MIT
//

use std::error::Error;
use std::fmt::Debug;
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
