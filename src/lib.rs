// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub mod client;
mod errors;
pub mod infrastructure;
pub mod location;
mod msg_id;
pub mod node;
mod serialisation;

pub use self::{
    client::ClientMessage,
    errors::{Error, Result},
    location::{DstLocation, SrcLocation, User},
    msg_id::MessageId,
    node::NodeMessage,
    serialisation::WireMsg,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Type of message
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    Ping,
    InfrastructureQuery(infrastructure::Query),
    ClientMessage(client::ClientMessage),
    NodeMessage(node::NodeMessage),
    RoutingMessage(node::RoutingMessage),
}

impl MessageType {
    /// serialize the message type into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        match self {
            Self::Ping => WireMsg::new_ping_msg().serialize(),
            Self::InfrastructureQuery(query) => WireMsg::serialize_infrastructure_query(query),
            Self::ClientMessage(msg) => WireMsg::serialize_client_msg(msg),
            Self::NodeMessage(msg) => WireMsg::serialize_node_msg(msg),
            Self::RoutingMessage(msg) => WireMsg::serialize_routing_msg(msg),
        }
    }
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Message {
    ///
    Client(ClientMessage),
    ///
    Node(NodeMessage),
}

impl Message {
    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::Node(msg) => msg.id(),
            Self::Client(msg) => msg.id(),
        }
    }
}
