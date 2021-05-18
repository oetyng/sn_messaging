// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

// FIXME: change NodeCmd defnintions to return Result and
// Error defined for the crate::node instead of client Result/Error
use crate::client::{CmdError, Error, Result};
use crate::{
    client::{
        BlobRead, BlobWrite, ClientSigned, DataCmd as NodeDataCmd, DataExchange,
        DataQuery as NodeDataQuery,
    },
    EndUser, MessageId, MessageType, WireMsg,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sn_data_types::{Blob, BlobAddress, NodeAge, PublicKey, SectionElders, Signature};
use std::collections::BTreeMap;
use xor_name::XorName;

// -------------- Node Cmd Messages --------------
// TODO: this messages hierarchy needs to be merged into
// the NodeMessage hierarchy. It's temporarily here till
// all messages defined within sn_routing are migrated to
// this crate and within NodeMessage struct.

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeMsg {
    /// Cmds only sent internally in the network.
    NodeCmd {
        /// NodeCmd.
        cmd: NodeCmd,
        /// Message ID.
        id: MessageId,
    },
    /// An error of a NodeCmd.
    NodeCmdError {
        /// The error.
        error: NodeCmdError,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Events only sent internally in the network.
    NodeEvent {
        /// Request.
        event: NodeEvent,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
    /// Queries is a read-only operation.
    NodeQuery {
        /// Query.
        query: NodeQuery,
        /// Message ID.
        id: MessageId,
    },
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// Message ID.
        id: MessageId,
        /// ID of causing query.
        correlation_id: MessageId,
    },
    /// The returned error, from any msg handling on recipient node.
    NodeMsgError {
        /// The error.
        error: Error,
        /// Message ID.
        id: MessageId,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
}

impl NodeMsg {
    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        match self {
            Self::NodeCmd { id, .. }
            | Self::NodeQuery { id, .. }
            | Self::NodeEvent { id, .. }
            | Self::NodeQueryResponse { id, .. }
            | Self::NodeCmdError { id, .. }
            | Self::NodeMsgError { id, .. } => *id,
        }
    }

    /// Convenience function to deserialize a 'NodeMsg' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node command message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::Node { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a node command message".to_string(),
            ))
        }
    }

    /// serialize this NodeCmd message into bytes ready to be sent over the wire.
    pub fn serialize(
        &self,
        dest: XorName,
        dest_section_pk: BlsPublicKey,
        src_section_pk: Option<BlsPublicKey>,
    ) -> crate::Result<Bytes> {
        WireMsg::serialize_node_msg(self, dest, dest_section_pk, src_section_pk)
    }
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmd {
    /// Metadata is handled by Elders
    Metadata {
        cmd: NodeDataCmd,
        client_signed: ClientSigned,
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    Chunks {
        cmd: BlobWrite,
        client_signed: ClientSigned,
        origin: EndUser,
    },
    /// Cmds related to the running of a node.
    System(NodeSystemCmd),
}

/// Cmds related to the running of a node.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemCmd {
    /// Register a wallet for rewards.
    RegisterWallet(PublicKey),
    /// Notify Elders on nearing max capacity
    StorageFull {
        /// Node Id
        node_id: PublicKey,
        /// Section to which the message needs to be sent to. (NB: this is the section of the node id).
        section: XorName,
    },
    /// Replicate a given chunk at an Adult
    ReplicateChunk(Blob),
    /// Tells the Elders to re-publish a chunk in the data section
    RepublishChunk(Blob),
    /// Sent to all promoted nodes (also sibling if any) after
    /// a completed transition to a new constellation.
    ReceiveExistingData {
        /// Age and wallets of registered nodes, keyed by node name.
        node_wallets: BTreeMap<XorName, (NodeAge, PublicKey)>,
        /// Metadata
        metadata: DataExchange,
    },
}

// -------------- Node Events --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    /// Replication completed event, emitted by a node, received by elders.
    ReplicationCompleted {
        ///
        chunk: BlobAddress,
        /// The Elder's accumulated signature
        /// over the chunk address. This is sent back
        /// to them so that any uninformed Elder knows
        /// that this is all good.
        proof: Signature,
    },
    /// Adults ack read/write of chunks as to convey responsivity.
    ChunkWriteHandled(Result<(), CmdError>),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQuery {
    /// Metadata is handled by Elders
    Metadata {
        query: NodeDataQuery,
        client_signed: ClientSigned,
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    Chunks { query: BlobRead, origin: EndUser },
    /// Related to the running of a node
    System(NodeSystemQuery),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemQuery {
    /// On Elder change, all Elders need to query
    /// network for the new wallet's replicas' public key set
    GetSectionElders,
    /// Acquire the chunk from current holders for replication.
    /// providing the address of the blob to be replicated.
    GetChunk(BlobAddress),
    /// The wallet a node has registered for rewards.
    GetNodeWalletKey(XorName),
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeSystemQueryResponse {
    /// On Elder change, all Elders need to query
    /// network for the new wallet's replicas' public key set
    GetSectionElders(SectionElders),
    /// Respond elders with the requested chunk for replication
    GetChunk(Blob),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQueryResponse {
    ///
    Data(NodeDataQueryResponse),
    ///
    System(NodeSystemQueryResponse),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataQueryResponse {
    /// Elder to Adult Get.
    GetChunk(Result<Blob>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmdError {
    ///
    Data(NodeDataError),
    ///
    Transfers(NodeTransferError),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeDataError {
    ///
    ChunkReplication {
        ///
        address: BlobAddress,
        ///
        error: Error,
    },
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeTransferError {
    /// The error of propagation of TransferRegistered event.
    TransferPropagation(Error),
}
