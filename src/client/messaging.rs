// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, QueryResponse, Result};
use crate::EndUser;
use serde::{Deserialize, Serialize};
use sn_data_types::{CreditAgreementProof as CreditProof, PublicKey, Token};
use std::collections::BTreeMap;
use xor_name::XorName;

pub type AgentId = u64;
pub type ClientId = XorName;
pub type GroupId = XorName;

// /// Agent to Group
// /// Any interaction an Agent has with
// /// a group, happens by sending a GPMGroupMsg.
// #[derive(Serialize)]
// pub struct GPMGroupMsg {
//     pub dst: GroupId,
//     pub msg_type: GPMGroupMsgType,
// }

// /// Group to Agent
// /// A GPMGroupMsg can have a response.
// pub struct GPMGroupMsgResponse {
//     pub src: GroupId,
//     pub dst: EndUser,
//     pub msg_type: GPMGroupMsgResponseType,
// }

/// Group to Agent
/// When an Agent sends a msg to another Agent,
/// it is sent as a GPMGroupMsg, and mapped at the
/// group into a MsgReceived event, which is sent to the recipient Agent.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct MsgReceived {
    pub src: GroupId,
    pub msg: GPMMsg,
}

/// Msgs for group interaction.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum GPMGroupQuery {
    GetConfig(GroupId),
}

impl GPMGroupQuery {
    /// Creates a QueryResponse containing an error, with the QueryResponse variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use GPMGroupQuery::*;
        match *self {
            GetConfig(_) => QueryResponse::GetGroupConfig(Err(error)),
        }
    }

    /// Returns the address of the destination for the query.
    pub fn dst_address(&self) -> XorName {
        use GPMGroupQuery::*;
        match self {
            GetConfig(group_id) => *group_id,
        }
    }
}

/// Msgs for group interaction.
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum GPMGroupCmd {
    SetOrCreate(GroupConfig),
    Join(AgentType),
    Leave(AgentType),
    Send { agent: u64, msg: GPMMsg },
    SendToAll(GPMMsg),
    Block { agent: u64, agent_type: AgentType },
    BlockAll(AgentType),
}

// a P2PComms msg
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct GPMMsg {
    // the msg type
    pub msg_type: u16,
    // unique name in the network
    pub group: GroupId,
    // if the type requires SNT payment
    pub payment: Option<CreditProof>,
    // the actual msg
    pub msg: Vec<u8>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Producer,
    Consumer,
    Both,
    Either,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    id: GroupId,
    name: String,
    owner: EndUser,
    msgs: BTreeMap<u16, MsgSettings>,
}

impl GroupConfig {
    ///
    pub fn new(name: String, owner: EndUser, msgs: BTreeMap<u16, MsgSettings>) -> Result<Self> {
        if msgs.is_empty() {
            return Err(Error::InvalidOperation);
        }

        Ok(Self {
            id: Self::id_from(&name)?,
            name,
            owner,
            msgs,
        })
    }

    ///
    pub fn id_from(name: &str) -> Result<GroupId> {
        if 3 > name.len() {
            return Err(Error::InvalidOperation);
        }
        Ok(XorName::from_content(&[
            "GroupId".as_bytes(),
            name.as_bytes(),
        ]))
    }

    ///
    pub fn id(&self) -> GroupId {
        self.id
    }

    ///
    pub fn name(&self) -> &str {
        &self.name
    }

    ///
    pub fn owner(&self) -> &EndUser {
        &self.owner
    }

    ///
    pub fn get_settings(&self, msg_type: &u16) -> Option<&MsgSettings> {
        self.msgs.get(msg_type)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct MsgSettings {
    /// group-unique type
    pub msg_type: u16,
    /// optional schema
    pub schema: Option<String>,
    /// if None, the type is free
    pub cost_scheme: CostScheme,
    /// can only be sent to this type
    pub sent_to: AgentType,
    /// can only be sent by this type
    pub sent_by: AgentType,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum CostScheme {
    // msgs are free
    None,
    // paid to the section
    Section(Token),
    /// paid to wallet
    Wallet {
        key: PublicKey,
        cost: Token,
    },
}
