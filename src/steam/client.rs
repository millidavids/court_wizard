//! Bevy wrapper around the Steamworks client: a `Client` resource and a message
//! carrying Steam callbacks. Vendored from `bevy-steamworks` 0.17, which has no
//! Bevy 0.20 release.
//!
//! Copyright 2021 James Liu
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to
//! deal in the Software without restriction, including without limitation the
//! rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
//! sell copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in
//! all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
//! IN THE SOFTWARE.

use std::ops::Deref;

use bevy::prelude::*;

/// The Steam client, dereferencing to [`steamworks::Client`].
#[derive(Resource, Clone)]
pub(crate) struct Client(steamworks::Client);

impl Client {
    pub(super) fn new(client: steamworks::Client) -> Self {
        Self(client)
    }
}

impl Deref for Client {
    type Target = steamworks::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Steam callback delivered by [`run_steam_callbacks`].
#[derive(Message, Debug)]
pub(crate) struct SteamCallbackMessage(pub(crate) steamworks::CallbackResult);

/// Pumps Steam callbacks into [`SteamCallbackMessage`]s.
pub(super) fn run_steam_callbacks(
    client: Res<Client>,
    mut output: MessageWriter<SteamCallbackMessage>,
) {
    client.process_callbacks(|callback| {
        output.write(SteamCallbackMessage(callback));
    });
}
