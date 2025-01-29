use crate::governance::track::Track;

pub struct VotingPolicy {
    pub participation_percent: u8,
    pub quorum_percent: u8,
    pub majority_percent: u8,
}

impl VotingPolicy {
    pub fn voting_policy_for_track(track: Track) -> Option<VotingPolicy> {
        match track {
            Track::Root => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            Track::WhitelistedCaller => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            Track::WishForChange => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            Track::Treasurer => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            Track::FellowshipAdmin => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            Track::SmallTipper => Some(Self {
                participation_percent: 30,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::BigTipper => Some(Self {
                participation_percent: 35,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::SmallSpender => Some(Self {
                participation_percent: 50,
                quorum_percent: 0,
                majority_percent: 50,
            }),
            Track::MediumSpender => Some(Self {
                participation_percent: 0,
                quorum_percent: 50,
                majority_percent: 50,
            }),
            Track::BigSpender => Some(Self {
                participation_percent: 0,
                quorum_percent: 60,
                majority_percent: 57,
            }),
            _ => None,
        }
    }
}
