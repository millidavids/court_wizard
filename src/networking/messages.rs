//! Networking Bevy messages.

use bevy::prelude::*;

/// Message sent when the P2P connection is successfully established.
#[derive(Message)]
#[allow(dead_code)]
pub struct ConnectionEstablishedMessage;

/// Message sent when the P2P connection is lost or fails.
#[derive(Message)]
#[allow(dead_code)]
pub struct ConnectionLostMessage;
