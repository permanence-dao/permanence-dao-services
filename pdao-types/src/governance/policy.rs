use crate::governance::track::Track;

pub fn round_half_down(x: f32) -> u32 {
    (x - 0.5).ceil() as u32
}

pub struct VotingPolicy {
    pub abstain_before_percent: f32,
    pub no_vote_before_percent: f32,
    pub majority_percent: f32,
}

impl VotingPolicy {
    pub fn voting_policy_for_track(track: &Track) -> VotingPolicy {
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
            | Track::BigSpender => Self {
                abstain_before_percent: 0.0,
                no_vote_before_percent: 50.0,
                majority_percent: 60.0,
            },
            Track::SmallTipper => Self {
                abstain_before_percent: 25.0,
                no_vote_before_percent: 0.0,
                majority_percent: 50.0,
            },
            Track::BigTipper | Track::SmallSpender => Self {
                abstain_before_percent: 37.5,
                no_vote_before_percent: 0.0,
                majority_percent: 50.0,
            },
            Track::MediumSpender => Self {
                abstain_before_percent: 0.0,
                no_vote_before_percent: 50.0,
                majority_percent: 50.0,
            },
        }
    }

    pub fn evaluate(
        &self,
        member_count: u32,
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
    ) -> VotingPolicyEvaluation {
        let participation = aye_count + nay_count + abstain_count;
        let abstain_threshold =
            round_half_down(self.abstain_before_percent * (member_count as f32) / 100.0);
        let participation_threshold =
            round_half_down(self.no_vote_before_percent * (member_count as f32) / 100.0);
        let aye_nay_majority_threshold =
            round_half_down(self.majority_percent * ((aye_count + nay_count) as f32) / 100.0);
        let total_majority_threshold =
            round_half_down(self.majority_percent * (participation as f32) / 100.0);
        let simple_majority_threshold = round_half_down(50.0 * (participation as f32) / 100.0);
        if participation < abstain_threshold {
            VotingPolicyEvaluation::AbstainThresholdNotMet {
                aye_count,
                nay_count,
                abstain_count,
                abstain_threshold,
            }
        } else if participation < participation_threshold {
            VotingPolicyEvaluation::ParticipationNotMet {
                aye_count,
                nay_count,
                abstain_count,
                participation_threshold,
            }
        } else if abstain_count > simple_majority_threshold {
            VotingPolicyEvaluation::MajorityAbstain {
                aye_count,
                nay_count,
                abstain_count,
                majority_threshold: simple_majority_threshold,
            }
        } else if abstain_count == 0 && aye_count == nay_count {
            VotingPolicyEvaluation::AyeEqualsNayAbstain {
                aye_count,
                nay_count,
                abstain_count,
            }
        } else if aye_count > aye_nay_majority_threshold {
            VotingPolicyEvaluation::Aye {
                aye_count,
                nay_count,
                abstain_count,
                majority_threshold: aye_nay_majority_threshold,
            }
        } else if (aye_count + abstain_count) > total_majority_threshold {
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count,
                nay_count,
                abstain_count,
                majority_threshold: total_majority_threshold,
            }
        } else {
            VotingPolicyEvaluation::Nay {
                aye_count,
                nay_count,
                abstain_count,
                majority_threshold: aye_nay_majority_threshold,
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum VotingPolicyEvaluation {
    AbstainThresholdNotMet {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        abstain_threshold: u32,
    },
    ParticipationNotMet {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        participation_threshold: u32,
    },
    MajorityAbstain {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        majority_threshold: u32,
    },
    AyeAbstainMajorityAbstain {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        majority_threshold: u32,
    },
    AyeEqualsNayAbstain {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
    },
    Aye {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        majority_threshold: u32,
    },
    Nay {
        aye_count: u32,
        nay_count: u32,
        abstain_count: u32,
        majority_threshold: u32,
    },
}

impl VotingPolicyEvaluation {
    pub fn get_aye_count(&self) -> u32 {
        match self {
            VotingPolicyEvaluation::AbstainThresholdNotMet { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::ParticipationNotMet { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::MajorityAbstain { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::AyeEqualsNayAbstain { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::Aye { aye_count, .. } => *aye_count,
            VotingPolicyEvaluation::Nay { aye_count, .. } => *aye_count,
        }
    }

    pub fn get_nay_count(&self) -> u32 {
        match self {
            VotingPolicyEvaluation::AbstainThresholdNotMet { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::ParticipationNotMet { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::MajorityAbstain { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::AyeEqualsNayAbstain { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::Aye { nay_count, .. } => *nay_count,
            VotingPolicyEvaluation::Nay { nay_count, .. } => *nay_count,
        }
    }

    pub fn get_abstain_count(&self) -> u32 {
        match self {
            VotingPolicyEvaluation::AbstainThresholdNotMet { abstain_count, .. } => *abstain_count,
            VotingPolicyEvaluation::ParticipationNotMet { abstain_count, .. } => *abstain_count,
            VotingPolicyEvaluation::MajorityAbstain { abstain_count, .. } => *abstain_count,
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain { abstain_count, .. } => {
                *abstain_count
            }
            VotingPolicyEvaluation::AyeEqualsNayAbstain { abstain_count, .. } => *abstain_count,
            VotingPolicyEvaluation::Aye { abstain_count, .. } => *abstain_count,
            VotingPolicyEvaluation::Nay { abstain_count, .. } => *abstain_count,
        }
    }

    pub fn simplify(&self) -> anyhow::Result<Option<bool>> {
        match self {
            VotingPolicyEvaluation::AbstainThresholdNotMet { .. } => Ok(None),
            VotingPolicyEvaluation::ParticipationNotMet { .. } => {
                anyhow::bail!("Outcome is no vote.")
            }
            VotingPolicyEvaluation::MajorityAbstain { .. } => Ok(None),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain { .. } => Ok(None),
            VotingPolicyEvaluation::AyeEqualsNayAbstain { .. } => Ok(None),
            VotingPolicyEvaluation::Aye { .. } => Ok(Some(true)),
            VotingPolicyEvaluation::Nay { .. } => Ok(Some(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_tipper() {
        let policy = VotingPolicy::voting_policy_for_track(&Track::SmallTipper);
        assert_eq!(
            policy.evaluate(8, 1, 0, 0),
            VotingPolicyEvaluation::AbstainThresholdNotMet {
                aye_count: 1,
                nay_count: 0,
                abstain_count: 0,
                abstain_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 0, 1, 0),
            VotingPolicyEvaluation::AbstainThresholdNotMet {
                aye_count: 0,
                nay_count: 1,
                abstain_count: 0,
                abstain_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 0, 0, 1),
            VotingPolicyEvaluation::AbstainThresholdNotMet {
                aye_count: 0,
                nay_count: 0,
                abstain_count: 1,
                abstain_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 0),
            VotingPolicyEvaluation::AyeEqualsNayAbstain {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 0,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 1),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 1,
                majority_threshold: 1,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 1, 3),
            VotingPolicyEvaluation::Aye {
                aye_count: 2,
                nay_count: 1,
                abstain_count: 3,
                majority_threshold: 1,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 2, 3),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 2,
                abstain_count: 3,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 3, 3),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 3,
                abstain_count: 3,
                majority_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 3, 3),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 1,
                nay_count: 3,
                abstain_count: 3,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 3, 2),
            VotingPolicyEvaluation::Nay {
                aye_count: 1,
                nay_count: 3,
                abstain_count: 2,
                majority_threshold: 2,
            },
        );
    }

    #[test]
    fn test_medium_spender() {
        let policy = VotingPolicy::voting_policy_for_track(&Track::MediumSpender);
        assert_eq!(
            policy.evaluate(8, 1, 0, 0),
            VotingPolicyEvaluation::ParticipationNotMet {
                aye_count: 1,
                nay_count: 0,
                abstain_count: 0,
                participation_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 1),
            VotingPolicyEvaluation::ParticipationNotMet {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 1,
                participation_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 2),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 2,
                majority_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 1, 2),
            VotingPolicyEvaluation::Aye {
                aye_count: 2,
                nay_count: 1,
                abstain_count: 2,
                majority_threshold: 1,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 3, 2),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 3,
                abstain_count: 2,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 4, 2),
            VotingPolicyEvaluation::Nay {
                aye_count: 2,
                nay_count: 4,
                abstain_count: 2,
                majority_threshold: 3,
            },
        );
    }

    #[test]
    fn test_big_spender() {
        let policy = VotingPolicy::voting_policy_for_track(&Track::BigSpender);
        assert_eq!(
            policy.evaluate(8, 1, 0, 0),
            VotingPolicyEvaluation::ParticipationNotMet {
                aye_count: 1,
                nay_count: 0,
                abstain_count: 0,
                participation_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 1),
            VotingPolicyEvaluation::ParticipationNotMet {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 1,
                participation_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 1, 1, 2),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 1,
                nay_count: 1,
                abstain_count: 2,
                majority_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 1, 1),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 1,
                abstain_count: 1,
                majority_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 1, 2),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 1,
                abstain_count: 2,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 2, 2),
            VotingPolicyEvaluation::Nay {
                aye_count: 2,
                nay_count: 2,
                abstain_count: 2,
                majority_threshold: 2,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 3, 1),
            VotingPolicyEvaluation::Nay {
                aye_count: 2,
                nay_count: 3,
                abstain_count: 1,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 3, 2),
            VotingPolicyEvaluation::Nay {
                aye_count: 2,
                nay_count: 3,
                abstain_count: 2,
                majority_threshold: 3,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 4, 2),
            VotingPolicyEvaluation::Nay {
                aye_count: 2,
                nay_count: 4,
                abstain_count: 2,
                majority_threshold: 4,
            },
        );
        assert_eq!(
            policy.evaluate(8, 2, 1, 3),
            VotingPolicyEvaluation::AyeAbstainMajorityAbstain {
                aye_count: 2,
                nay_count: 1,
                abstain_count: 3,
                majority_threshold: 4,
            },
        );
    }
}
