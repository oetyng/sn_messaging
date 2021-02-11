// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub mod network;
pub mod routing;

use crate::{Error, MessageId, MessageType, Result, SrcLocation, WireMsg};
use bytes::Bytes;
pub use network::*;
pub use routing::RoutingMessage;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Node-to-Node comms back and forth
#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum NodeMessage {
    /// Cmds only sent internally in the network.
    NodeCmd {
        /// NodeCmd.
        cmd: NodeCmd,
        /// Message ID.
        id: MessageId,
        /// Target section's current PublicKey
        target_section_pk: Option<PublicKey>,
    },
    /// An error of a NodeCmd.
    NodeCmdError {
        /// The error.
        error: NodeCmdError,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
        /// The sender of the causing cmd.
        cmd_origin: SrcLocation,
        /// Target section's current PublicKey
        target_section_pk: Option<PublicKey>,
    },
    /// Events only sent internally in the network.
    NodeEvent {
        /// Request.
        event: NodeEvent,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
        /// Target section's current PublicKey
        target_section_pk: Option<PublicKey>,
    },
    /// Queries is a read-only operation.
    NodeQuery {
        /// Query.
        query: NodeQuery,
        /// Message ID.
        id: MessageId,
        /// Target section's current PublicKey
        target_section_pk: Option<PublicKey>,
    },
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// Message ID.
        id: MessageId,
        /// ID of causing query.
        correlation_id: MessageId,
        /// The sender of the causing query.
        query_origin: SrcLocation,
        /// Target section's current PublicKey
        target_section_pk: Option<PublicKey>,
    },
}

// /// Node message sent over the network.
// // TODO: this is currently holding just bytes as a placeholder, next step
// // is to move all actual node messages structs and definitions within it.
// #[derive(Clone, Eq, Serialize, Deserialize)]
// pub struct NodeMessage(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl NodeMessage {
    /// Convinience function to deserialize a 'NodeMessage' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node message.
    pub fn from(bytes: Bytes) -> Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::NodeMessage(msg) = deserialized {
            Ok(msg)
        } else {
            Err(Error::FailedToParse("bytes as a node message".to_string()))
        }
    }

    /// serialize this NodeMessage into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> Result<Bytes> {
        WireMsg::serialize_node_msg(self)
    }

    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::NodeCmd { id, .. }
            | Self::NodeEvent { id, .. }
            | Self::NodeQuery { id, .. }
            | Self::NodeCmdError { id, .. }
            | Self::NodeQueryResponse { id, .. } => *id,
        }
    }
}

impl Into<crate::Message> for NodeMessage {
    fn into(self) -> crate::Message {
        crate::Message::Node(self)
    }
}
