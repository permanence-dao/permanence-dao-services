use crate::governance::track::Track;

pub struct VotingPolicy {
    pub abstain_threshold_percent: u8,
    pub participation_percent: u8,
    pub quorum_percent: u8,
    pub majority_percent: u8,
}

impl VotingPolicy {
    pub fn voting_policy_for_track(track: &Track) -> Option<VotingPolicy> {
        match track {
            Track::Root
            | Track::WhitelistedCaller
            | Track::WishForChange
            | Track::Treasurer
            | Track::FellowshipAdmin
            | Track::StakingAdmin
            | Track::LeaseAdmin
            | Track::GeneralAdmin
            | Track::AuctionAdmin
            | Track::ReferendumCanceller
            | Track::ReferendumKiller
            | Track::BigSpender => Some(Self {
                abstain_threshold_percent: 50,
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 0,
            }),
            Track::SmallTipper => Some(Self {
                abstain_threshold_percent: 50,
                participation_percent: 30,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::BigTipper => Some(Self {
                abstain_threshold_percent: 50,
                participation_percent: 35,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::SmallSpender => Some(Self {
                abstain_threshold_percent: 50,
                participation_percent: 50,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::MediumSpender => Some(Self {
                abstain_threshold_percent: 50,
                participation_percent: 0,
                quorum_percent: 50,
                majority_percent: 50,
            }),
        }
    }
}
