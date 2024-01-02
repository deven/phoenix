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

macro_rules! actor_field_constructor_param_decl {
    ( $field_name:ident ( ) : ( $field_type:ty ) ) => {
        $field_name: Option<$field_type>,
    };
    ( $field_name:ident ( ) : ( $field_type:ty, $( $not_found:ident )? ) ) => {
        $field_name: $field_type,
    };
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( $field_type:ty $( , $not_found:ident )? ) ) => {
        /* empty */
    };
}

macro_rules! actor_field_constructor_param_value {
    ( $field_name:ident ( ) : ( $field_type:ty, $( $not_found:ident )? ) ) => {
        $field_name,
    };
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( $field_type:ty $( , $not_found:ident )? ) ) => {
    };
}

macro_rules! actor_field_getter {
    ( $field_name:ident : ( String ), $error:ident ) => {
        pub fn $field_name(&self) -> Result<Option<Arc<str>>, $error> {
            self.state_rx.borrow().$field_name.clone()
        }
    };
    ( $field_name:ident : ( String, $not_found:ident ), $error:ident ) => {
        pub fn $field_name(&self) -> Result<Arc<str>, $error> {
            self.state_rx
                .borrow()
                .$field_name
                .clone()
                .ok_or($error::$not_found)
        }
    };
    ( $field_name:ident : ( $field_type:ty ), $error:ident ) => {
        pub fn $field_name(&self) -> Result<Option<$field_type>, $error> {
            self.state_rx.borrow().$field_name.clone()
        }
    };
    ( $field_name:ident : ( $field_type:ty, $not_found:ident ), $error:ident ) => {
        pub fn $field_name(&self) -> Result<$field_type, $error> {
            self.state_rx
                .borrow()
                .$field_name
                .clone()
                .ok_or($error::$not_found)
        }
    };
}

macro_rules! actor_field_setter {
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( String $( , $not_found:ident )? ) ) => {
        #[framed]
        pub async fn $setter(
            &self,
            username: Option<String>,
        ) -> Result<Option<Arc<str>>, SessionError> {
            let (response_tx, response_rx) = oneshot::channel();
            self.actor_tx
                .send($msg::$msg_variant(response_tx, $field_name))
                .await?;
            response_rx.await?
        }
    };
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( $field_type:ty $( , $not_found:ident )? ) ) => {
        #[framed]
        pub async fn $setter(
            &self,
            username: Option<$field_type>,
        ) -> Result<Option<$field_type>, SessionError> {
            let (response_tx, response_rx) = oneshot::channel();
            self.actor_tx
                .send($msg::$msg_variant(response_tx, $field_name))
                .await?;
            response_rx.await?
        }
    };
    ( $field_name:ident ( ) : ( String $( , $not_found:ident )? ) ) => {
    };
}

macro_rules! actor_field {
    ( $field_name:ident : ( String, $( $not_found:ident )? ) ) => {
        pub $field_name: Option<Arc<str>>,
    };
    ( $field_name:ident : ( $field_type:ty, $( $not_found:ident )? ) ) => {
        pub $field_name: Option<$field_type>,
    };
}

macro_rules! actor_field_handle_message {
    ( $msg:ident, $field_name:ident ( $setter:ident, $msg_variant:ident ) ) => {
        $msg::$msg_variant(respond_to, $field_name) => {
            let _ = respond_to.send(self.$setter($field_name));
        },
    };
    ( $msg:ident, $field_name:ident ( ) ) => {
    };
}

macro_rules! actor_field_update_func {
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( String $( , $not_found:ident )? ) ) => {
        fn $setter(&mut self, new_value: Option<String>) -> Result<Option<Arc<str>>, SessionError> {
            let new_value = new_value.map(|s| Arc::from(s.into_boxed_str()));

            Arc::make_mut(&mut self.state).$field_name = new_value.clone();
            self.state = self.state_tx.send_replace(self.state.clone());

            let old_value = self.state.$field_name.clone();
            Arc::make_mut(&mut self.state).$field_name = new_value;

            Ok(old_value)
        }
    };
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( $field_type:ty $( , $not_found:ident )? ) ) => {
        fn $setter(
            &mut self,
            new_value: Option<$field_type>,
        ) -> Result<Option<Arc<str>>, SessionError> {
            Arc::make_mut(&mut self.state).$field_name = new_value.clone();
            self.state = self.state_tx.send_replace(self.state.clone());

            let old_value = self.state.$field_name.clone();
            Arc::make_mut(&mut self.state).$field_name = new_value;

            Ok(old_value)
        }
    };
    ( $field_name:ident ( ) : ( $field_type:ty $( , $not_found:ident )? ) ) => {
    };
}

macro_rules! actor_field_msg_variant {
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( String $( , $not_found:ident )? ), $error:ident ) => {
        $msg_variant(
            oneshot::Sender<Result<Option<Arc<str>>, $error>>,
            Option<$field_type>,
        ),
    };
    ( $field_name:ident ( $setter:ident, $msg_variant:ident ) : ( $field_type:ty $( , $not_found:ident )? ), $error:ident ) => {
        $msg_variant(
            oneshot::Sender<Result<Option<$field_type>, $error>>,
            Option<$field_type>,
        ),
    };
    ( $field_name:ident ( ) : ( $field_type:ty $( , $not_found:ident )? ), $error:ident ) => {
    };
}

macro_rules! actor_field_error_variant {
    ( ( $field_type:ty, $not_found:ident ) ) => {
        $not_found,
    };
    ( $field_type:ty ) => {
    };
}

macro_rules! actor_error_enum {
    ( $error:ident, $msg:ident, $( ( $field_type:ty $( , $not_found:ident )? ) )* ) => {
        #[derive(Debug)]
        pub enum $error {
            TxError(mpsc::error::SendError<$msg>),
            RxError(oneshot::error::RecvError),
            $(
                $( $not_found, )?
            )*
        }
    };
}

macro_rules! actor_field_error_source {
    ( ( $field_type:ty, $not_found:ident ) ) => {
        Self::$not_found => None,
    };
    ( $field_type:ty ) => {
    };
}

macro_rules! actor_field_error_fmt {
    ( ( $field_type:ty, $not_found:ident ) ) => {
        Self::$not_found => write!(f, "{} not found!", stringify!($field_name)),
    };
    ( $field_type:ty ) => {
    };
}

macro_rules! create_actor {
    ( $actor:ident, $inner:ident, $state:ident, $msg:ident, $error:ident => $( $field_name:ident $setter:tt : $field_type:tt ),* ) => {
        #[derive(Debug, Clone)]
        pub struct $actor {
            actor_tx: mpsc::Sender<$msg>,
            state_rx: watch::Receiver<Arc<$state>>,
        }

        impl $actor {
            pub fn new(
//                $(
//                    $crate::actor_field_constructor_param_decl!($field_name $setter : $field_type)
//                )*
            ) -> Self {
                let (inner, actor_tx, state_rx) = $inner::new(
//                    $(
//                        $crate::actor_field_constructor_param_value!($field_name $setter : $field_type)
//                    )*
                );
                tokio::spawn(frame!(async move { ActorInner::run(inner).await }));
                Self { actor_tx, state_rx }
            }

            $(
                actor_field_getter!($field_name : $field_type, $error);
//                $crate::actor_field_setter!($field_name $setter : $field_type);
            )*
        }

        impl $crate::actor::Actor for $actor {
            type Error = $error;
        }

        #[derive(Debug, Clone, Default)]
        pub struct $state {
//            $(
//                $crate::actor_field!($field_name : $field_type),
//            )*
        }

        impl $state {
            pub fn new(
//                $(
//                    $crate::actor_field_constructor_param_decl!($field_name $setter : $field_type)
//                )*
            ) -> Self {
                Self {
//                    $(
//                        $crate::actor_field_constructor_param_value!($field_name $setter : $field_type)
//                    )*
                }
            }
        }

        #[derive(Debug)]
        struct $inner {
            actor_rx: mpsc::Receiver<$msg>,
            state_tx: watch::Sender<Arc<$state>>,
            state: Arc<$state>,
        }

        impl $inner {
            pub fn new() -> (
                Self,
                mpsc::Sender<$msg>,
                watch::Receiver<Arc<$state>>,
            ) {
                let state = Arc::from($state::new());

                let (actor_tx, actor_rx) = mpsc::channel(8);
                let (state_tx, state_rx) = watch::channel(state.clone());

                let inner = Self {
                    actor_rx,
                    state_tx,
                    state,
                };

                (inner, actor_tx, state_rx)
            }

            #[framed]
            async fn handle_message(&mut self, msg: $msg) -> Result<(), $error> {
//                match msg {
//                    $(
//                        $crate::actor_field_handle_message!($msg, $field_name $setter)
//                    )*
//                };

                Ok(())
            }

//            $(
//                $crate::actor_field_update_func!($field_name $setter : $field_type);
//            )*
        }

        impl $crate::actor::ActorInner for $inner {
            type Error = $error;

            #[framed]
            async fn run(mut self) -> Result<(), Self::Error>
            where
                Self: Sized,
            {
                while let Some(msg) = self.actor_rx.recv().await {
                    let debug_msg = format!("{msg:?}");
                    if let Err(e) = self.handle_message(msg).await {
                        warn!("Error handling {debug_msg}: {e:?}");
                    }
                }
                Ok(())
            }
        }

        #[derive(Debug)]
        pub enum $msg {
//            $(
//                $crate::actor_field_msg_variant!($field_name $setter : $field_type, $error)
//            )*
        }

        actor_error_enum!($error, $msg, $( $field_type )*);

        impl std::error::Error for $error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    Self::TxError(err) => err.source(),
                    Self::RxError(err) => err.source(),
//                    $(
//                        $crate::actor_field_error_source!($field_type)
//                    )*
                }
            }
        }

        impl std::fmt::Display for $error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::TxError(err) => std::fmt::Display::fmt(&err, f),
                    Self::RxError(err) => std::fmt::Display::fmt(&err, f),
//                    $(
//                        $crate::actor_field_error_fmt!($field_type)
//                    )*
                }
            }
        }

        impl From<mpsc::error::SendError<$msg>> for $error {
            fn from(err: mpsc::error::SendError<$msg>) -> Self {
                Self::TxError(err)
            }
        }

        impl From<oneshot::error::RecvError> for $error {
            fn from(err: oneshot::error::RecvError) -> Self {
                Self::RxError(err)
            }
        }
    };
}

create_actor!(Session, SessionInner, SessionState, SessionMsg, SessionError =>
    username(set_username, SetUsername): (String, UsernameNotFound)
);

//create_actor!(Client, ClientInner, ClientState, ClientMsg, ClientError =>
//    session(set_session, SetSession): (Session, SocketAddrNotFound),
//    addr(): (SocketAddr)
//);
