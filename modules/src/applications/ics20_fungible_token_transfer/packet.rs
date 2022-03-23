use alloc::string::ToString;
use core::convert::TryFrom;
use core::str::FromStr;

use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData as RawPacketData;

use super::error::Error;
use super::{Coin, Decimal, Denom};
use crate::signer::Signer;

#[derive(Clone, Debug, PartialEq)]
pub struct PacketData {
    pub token: Coin,
    pub sender: Signer,
    pub receiver: Signer,
}

impl TryFrom<RawPacketData> for PacketData {
    type Error = Error;

    fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
        let denom = Denom::from_str(&raw_pkt_data.denom)?;
        let amount = Decimal::from_str(&raw_pkt_data.amount).map_err(Error::invalid_coin_amount)?;
        Ok(Self {
            token: Coin { denom, amount },
            sender: raw_pkt_data.sender.into(),
            receiver: raw_pkt_data.receiver.into(),
        })
    }
}

impl From<PacketData> for RawPacketData {
    fn from(pkt_data: PacketData) -> Self {
        Self {
            denom: pkt_data.token.denom.to_string(),
            amount: pkt_data.token.amount.to_string(),
            sender: pkt_data.sender.to_string(),
            receiver: pkt_data.receiver.to_string(),
        }
    }
}
