use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Sequence, Serialize, Deserialize)]
pub enum Track {
    Root,
    WhitelistedCaller,
    WishForChange,
    // general admin
    StakingAdmin,
    Treasurer,
    LeaseAdmin,
    FellowshipAdmin,
    GeneralAdmin,
    AuctionAdmin,
    // referendum admins
    ReferendumCanceller,
    ReferendumKiller,
    // limited treasury spenders
    SmallTipper,
    BigTipper,
    SmallSpender,
    MediumSpender,
    BigSpender,
}

impl Track {
    pub fn id(&self) -> u16 {
        match self {
            Track::Root => 0,
            Track::WhitelistedCaller => 1,
            Track::WishForChange => 2,
            // general admin
            Track::StakingAdmin => 10,
            Track::Treasurer => 11,
            Track::LeaseAdmin => 12,
            Track::FellowshipAdmin => 13,
            Track::GeneralAdmin => 14,
            Track::AuctionAdmin => 15,
            // referendum admins
            Track::ReferendumCanceller => 20,
            Track::ReferendumKiller => 21,
            // limited treasury spenders
            Track::SmallTipper => 30,
            Track::BigTipper => 31,
            Track::SmallSpender => 32,
            Track::MediumSpender => 33,
            Track::BigSpender => 34,
        }
    }

    pub fn short_name(&self) -> &str {
        match self {
            Track::Root => "R",
            Track::WhitelistedCaller => "WC",
            Track::WishForChange => "WFC",
            // general admin
            Track::StakingAdmin => "SA",
            Track::Treasurer => "T",
            Track::LeaseAdmin => "LA",
            Track::FellowshipAdmin => "FA",
            Track::GeneralAdmin => "GA",
            Track::AuctionAdmin => "AA",
            // referendum admins
            Track::ReferendumCanceller => "RC",
            Track::ReferendumKiller => "RK",
            // limited treasury spenders
            Track::SmallTipper => "ST",
            Track::BigTipper => "BT",
            Track::SmallSpender => "SS",
            Track::MediumSpender => "MS",
            Track::BigSpender => "BS",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Track::Root => "Root",
            Track::WhitelistedCaller => "Whitelisted Caller",
            Track::WishForChange => "Wish For Change",
            // general admin
            Track::StakingAdmin => "Staking Admin",
            Track::Treasurer => "Treasurer",
            Track::LeaseAdmin => "Lease Admin",
            Track::FellowshipAdmin => "Fellowship Admin",
            Track::GeneralAdmin => "General Admin",
            Track::AuctionAdmin => "Auction Admin",
            // referendum admins
            Track::ReferendumCanceller => "Referendum Canceller",
            Track::ReferendumKiller => "Referendum Killer",
            // limited treasury spenders
            Track::SmallTipper => "Small Tipper",
            Track::BigTipper => "Big Tipper",
            Track::SmallSpender => "Small Spender",
            Track::MediumSpender => "Medium Spender",
            Track::BigSpender => "Big Spender",
        }
    }

    pub fn from_id(id: u16) -> Option<Track> {
        match id {
            0 => Some(Track::Root),
            1 => Some(Track::WhitelistedCaller),
            2 => Some(Track::WishForChange),
            10 => Some(Track::StakingAdmin),
            11 => Some(Track::Treasurer),
            12 => Some(Track::LeaseAdmin),
            13 => Some(Track::FellowshipAdmin),
            14 => Some(Track::GeneralAdmin),
            15 => Some(Track::AuctionAdmin),
            20 => Some(Track::ReferendumCanceller),
            21 => Some(Track::ReferendumKiller),
            30 => Some(Track::SmallTipper),
            31 => Some(Track::BigTipper),
            32 => Some(Track::SmallSpender),
            33 => Some(Track::MediumSpender),
            34 => Some(Track::BigSpender),
            _ => None,
        }
    }
}
