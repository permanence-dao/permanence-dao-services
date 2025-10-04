use crate::governance::track::Track;

pub fn round_half_down(x: f32) -> u32 {
    (x - 0.5).ceil() as u32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoteCounts {
    members: u32,
    ayes: u32,
    nays: u32,
    abstains: u32,
}

impl VoteCounts {
    pub fn new(members: u32, ayes: u32, nays: u32, abstains: u32) -> Self {
        Self {
            members,
            ayes,
            nays,
            abstains,
        }
    }

    pub fn participation(&self) -> u32 {
        self.ayes
            .saturating_add(self.nays)
            .saturating_add(self.abstains)
    }
}

pub enum ParticipationRequirement {
    AbstainBeforePercent(Comparison),
    NoVoteBeforePercent(Comparison),
}

pub enum MajorityNominator {
    Ayes,
    Nays,
    Abstains,
}

impl MajorityNominator {
    pub fn get(&self, vote_counts: &VoteCounts) -> u32 {
        match self {
            MajorityNominator::Ayes => vote_counts.ayes,
            MajorityNominator::Nays => vote_counts.nays,
            MajorityNominator::Abstains => vote_counts.abstains,
        }
    }
}

pub enum MajorityDenominator {
    OfNonAbstainVotes,
    OfAllVotes,
}

impl MajorityDenominator {
    pub fn get(&self, vote_counts: &VoteCounts) -> u32 {
        match self {
            MajorityDenominator::OfNonAbstainVotes => {
                vote_counts.ayes.saturating_add(vote_counts.nays)
            }
            MajorityDenominator::OfAllVotes => vote_counts
                .ayes
                .saturating_add(vote_counts.nays)
                .saturating_add(vote_counts.abstains),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Comparison {
    GreaterThan(f32),
    GreaterThanOrEqual(f32),
}

impl Comparison {
    pub fn holds(&self, value: f32) -> bool {
        match self {
            Comparison::GreaterThan(threshold) => value > *threshold,
            Comparison::GreaterThanOrEqual(threshold) => value >= *threshold,
        }
    }

    pub fn threshold_rate(&self) -> f32 {
        match self {
            Comparison::GreaterThan(threshold) => *threshold,
            Comparison::GreaterThanOrEqual(threshold) => *threshold,
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Comparison::GreaterThan(_) => ">".to_string(),
            Comparison::GreaterThanOrEqual(_) => "≥".to_string(),
        }
    }
}

pub struct Policy {
    participation_requirement: ParticipationRequirement,
    majority_nominator: MajorityNominator,
    majority_comparison: Comparison,
    majority_denominator: MajorityDenominator,
}

impl Policy {
    pub fn policy_for_track(track: &Track) -> Self {
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
                participation_requirement: ParticipationRequirement::NoVoteBeforePercent(
                    Comparison::GreaterThanOrEqual(50.0),
                ),
                majority_nominator: MajorityNominator::Ayes,
                majority_comparison: Comparison::GreaterThanOrEqual(60.0),
                majority_denominator: MajorityDenominator::OfAllVotes,
            },
            Track::MediumSpender => Self {
                participation_requirement: ParticipationRequirement::NoVoteBeforePercent(
                    Comparison::GreaterThanOrEqual(50.0),
                ),
                majority_nominator: MajorityNominator::Ayes,
                majority_comparison: Comparison::GreaterThan(50.0),
                majority_denominator: MajorityDenominator::OfAllVotes,
            },
            Track::SmallSpender | Track::BigTipper => Self {
                participation_requirement: ParticipationRequirement::AbstainBeforePercent(
                    Comparison::GreaterThanOrEqual(37.5),
                ),
                majority_nominator: MajorityNominator::Ayes,
                majority_comparison: Comparison::GreaterThan(50.0),
                majority_denominator: MajorityDenominator::OfNonAbstainVotes,
            },
            Track::SmallTipper => Self {
                participation_requirement: ParticipationRequirement::AbstainBeforePercent(
                    Comparison::GreaterThanOrEqual(25.0),
                ),
                majority_nominator: MajorityNominator::Ayes,
                majority_comparison: Comparison::GreaterThan(50.0),
                majority_denominator: MajorityDenominator::OfNonAbstainVotes,
            },
        }
    }

    pub fn evaluate(&self, vote_counts: VoteCounts) -> (PolicyEvaluation, Vec<String>) {
        let participation_percent =
            (vote_counts.participation() as f32) * 100.0 / (vote_counts.members as f32);
        let simple_majority_threshold = 50.0 * (vote_counts.participation() as f32) / 100.0;
        let mut description_lines = Vec::new();
        match &self.participation_requirement {
            ParticipationRequirement::AbstainBeforePercent(comparison) => {
                let line = format!(
                    "Abstain before {}{:.1}% participation.",
                    comparison.symbol(),
                    comparison.threshold_rate(),
                );
                if !comparison.holds(participation_percent) {
                    description_lines.push(format!("▶️ {}", line));
                    description_lines.push("⚪ ABSTAIN".to_string());
                    return (
                        PolicyEvaluation::AbstainThresholdNotMet {
                            vote_counts,
                            abstain_threshold: (vote_counts.members as f32)
                                * comparison.threshold_rate()
                                / 100.0,
                        },
                        description_lines,
                    );
                } else {
                    description_lines.push(format!("✔️ {}", line));
                }
            }
            ParticipationRequirement::NoVoteBeforePercent(comparison) => {
                let line = format!(
                    "No vote before {}{:.1}% participation.",
                    comparison.symbol(),
                    comparison.threshold_rate(),
                );
                if !comparison.holds(participation_percent) {
                    description_lines.push(format!("▶️ {}", line));
                    description_lines.push("➖ NO VOTE".to_string());
                    return (
                        PolicyEvaluation::ParticipationNotMet {
                            vote_counts,
                            participation_threshold: (vote_counts.members as f32)
                                * comparison.threshold_rate()
                                / 100.0,
                        },
                        description_lines,
                    );
                } else {
                    description_lines.push(format!("✔️ {}", line));
                }
            }
        }

        let majority_nominator = self.majority_nominator.get(&vote_counts) as f32;
        let majority_denominator = self.majority_denominator.get(&vote_counts) as f32;
        let non_aye_percent = (vote_counts.ayes + vote_counts.abstains) as f32 * 100.0
            / vote_counts.participation() as f32;
        let majority_percent = majority_nominator * 100.0 / majority_denominator;
        let majority_threshold =
            self.majority_comparison.threshold_rate() * majority_denominator / 100.0;

        let majority_abstain_line = "Abstain if >50% all votes are abstain.";
        if (vote_counts.abstains as f32) > simple_majority_threshold {
            description_lines.push(format!("▶️ {}", majority_abstain_line));
            description_lines.push("⚪ ABSTAIN".to_string());
            return (
                PolicyEvaluation::MajorityAbstain {
                    vote_counts,
                    majority_threshold: simple_majority_threshold,
                },
                description_lines,
            );
        } else {
            description_lines.push(format!("✔️ {}", majority_abstain_line));
        }

        let ayes_equal_nays_line = "Abstain if ayes are equal to nays with no abstains.";
        if vote_counts.abstains == 0 && (vote_counts.ayes == vote_counts.nays) {
            description_lines.push(format!("▶️ {}", ayes_equal_nays_line));
            description_lines.push("⚪ ABSTAIN".to_string());
            return (
                PolicyEvaluation::AyeEqualsNayAbstain { vote_counts },
                description_lines,
            );
        } else {
            description_lines.push(format!("✔️ {}", ayes_equal_nays_line));
        }

        let aye_line = format!(
            "Aye if {}{:.1}% ayes out of {} votes.",
            self.majority_comparison.symbol(),
            self.majority_comparison.threshold_rate(),
            match self.majority_denominator {
                MajorityDenominator::OfNonAbstainVotes => "non-abstain",
                MajorityDenominator::OfAllVotes => "all",
            },
        );
        if self.majority_comparison.holds(majority_percent) {
            description_lines.push(format!("▶️ {}", aye_line));
            description_lines.push("🟢 AYE".to_string());
            return (
                PolicyEvaluation::Aye {
                    vote_counts,
                    majority_threshold,
                },
                description_lines,
            );
        } else {
            description_lines.push(format!("❌ {}", aye_line));
        }

        let aye_abstain_majority_line = format!(
            "Abstain if ayes and abstains are {}{:.1}% of all votes.",
            self.majority_comparison.symbol(),
            self.majority_comparison.threshold_rate(),
        );
        if self.majority_comparison.holds(non_aye_percent) {
            description_lines.push(format!("▶️ {}", aye_abstain_majority_line));
            description_lines.push("⚪ ABSTAIN".to_string());
            return (
                PolicyEvaluation::AyeAbstainMajorityAbstain {
                    vote_counts,
                    majority_threshold: self.majority_comparison.threshold_rate()
                        * vote_counts.participation() as f32
                        / 100.0,
                },
                description_lines,
            );
        } else {
            description_lines.push(format!("❌ {}", aye_abstain_majority_line));
        }
        description_lines.push("🔴 NAY".to_string());
        (
            PolicyEvaluation::Nay {
                vote_counts,
                majority_threshold,
            },
            description_lines,
        )
    }

    pub fn get_description(&self, vote_counts: &VoteCounts) -> String {
        let mut lines = Vec::new();
        match &self.participation_requirement {
            ParticipationRequirement::AbstainBeforePercent(comparison) => lines.push(format!(
                "Abstain before {}{:.1}% participation is met ({}{:.1} out of {} members).",
                comparison.symbol(),
                comparison.threshold_rate(),
                comparison.symbol(),
                comparison.threshold_rate() * vote_counts.members as f32 / 100.0,
                vote_counts.members,
            )),
            ParticipationRequirement::NoVoteBeforePercent(comparison) => lines.push(format!(
                "No vote before {}{:.1}% participation is met ({}{:.1} out of {} members).",
                comparison.symbol(),
                comparison.threshold_rate(),
                comparison.symbol(),
                comparison.threshold_rate() * vote_counts.members as f32 / 100.0,
                vote_counts.members,
            )),
        }
        lines.push("Abstain if ayes are equal to nays without any abstain votes.".to_string());
        lines.push("Abstain if >50% all votes are abstain.".to_string());
        lines.push(format!(
            "Abstain if ayes and abstains are {}{:.1}% of all votes.",
            self.majority_comparison.symbol(),
            self.majority_comparison.threshold_rate(),
        ));
        lines.push(format!(
            "Aye if {}{:.1}% are aye out of of {} votes.",
            self.majority_comparison.symbol(),
            self.majority_comparison.threshold_rate(),
            match self.majority_denominator {
                MajorityDenominator::OfNonAbstainVotes => "non-abstain",
                MajorityDenominator::OfAllVotes => "all",
            },
        ));
        lines.push("Nay if none of the above cases above hold.".to_string());
        lines.join("\n")
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PolicyEvaluation {
    AbstainThresholdNotMet {
        vote_counts: VoteCounts,
        abstain_threshold: f32,
    },
    ParticipationNotMet {
        vote_counts: VoteCounts,
        participation_threshold: f32,
    },
    MajorityAbstain {
        vote_counts: VoteCounts,
        majority_threshold: f32,
    },
    AyeAbstainMajorityAbstain {
        vote_counts: VoteCounts,
        majority_threshold: f32,
    },
    AyeEqualsNayAbstain {
        vote_counts: VoteCounts,
    },
    Aye {
        vote_counts: VoteCounts,
        majority_threshold: f32,
    },
    Nay {
        vote_counts: VoteCounts,
        majority_threshold: f32,
    },
}

impl PolicyEvaluation {
    pub fn get_aye_count(&self) -> u32 {
        match self {
            PolicyEvaluation::AbstainThresholdNotMet { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::ParticipationNotMet { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::MajorityAbstain { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::AyeAbstainMajorityAbstain { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::AyeEqualsNayAbstain { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::Aye { vote_counts, .. } => vote_counts.ayes,
            PolicyEvaluation::Nay { vote_counts, .. } => vote_counts.ayes,
        }
    }

    pub fn get_nay_count(&self) -> u32 {
        match self {
            PolicyEvaluation::AbstainThresholdNotMet { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::ParticipationNotMet { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::MajorityAbstain { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::AyeAbstainMajorityAbstain { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::AyeEqualsNayAbstain { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::Aye { vote_counts, .. } => vote_counts.nays,
            PolicyEvaluation::Nay { vote_counts, .. } => vote_counts.nays,
        }
    }

    pub fn get_abstain_count(&self) -> u32 {
        match self {
            PolicyEvaluation::AbstainThresholdNotMet { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::ParticipationNotMet { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::MajorityAbstain { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::AyeAbstainMajorityAbstain { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::AyeEqualsNayAbstain { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::Aye { vote_counts, .. } => vote_counts.abstains,
            PolicyEvaluation::Nay { vote_counts, .. } => vote_counts.abstains,
        }
    }

    pub fn simplify(&self) -> anyhow::Result<Option<bool>> {
        match self {
            PolicyEvaluation::AbstainThresholdNotMet { .. } => Ok(None),
            PolicyEvaluation::ParticipationNotMet { .. } => {
                anyhow::bail!("Outcome is no vote.")
            }
            PolicyEvaluation::MajorityAbstain { .. } => Ok(None),
            PolicyEvaluation::AyeAbstainMajorityAbstain { .. } => Ok(None),
            PolicyEvaluation::AyeEqualsNayAbstain { .. } => Ok(None),
            PolicyEvaluation::Aye { .. } => Ok(Some(true)),
            PolicyEvaluation::Nay { .. } => Ok(Some(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_tipper() {
        let policy = Policy::policy_for_track(&Track::SmallTipper);
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 0, 0)).0,
            PolicyEvaluation::AbstainThresholdNotMet {
                vote_counts: VoteCounts::new(8, 1, 0, 0),
                abstain_threshold: 2.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 0, 1, 0)).0,
            PolicyEvaluation::AbstainThresholdNotMet {
                vote_counts: VoteCounts::new(8, 0, 1, 0),
                abstain_threshold: 2.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 0, 0, 1)).0,
            PolicyEvaluation::AbstainThresholdNotMet {
                vote_counts: VoteCounts::new(8, 0, 0, 1),
                abstain_threshold: 2.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 0)).0,
            PolicyEvaluation::AyeEqualsNayAbstain {
                vote_counts: VoteCounts::new(8, 1, 1, 0),
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 1)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 1, 1),
                majority_threshold: 1.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 1, 3)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 2, 1, 3),
                majority_threshold: 1.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 2, 3)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 2, 3),
                majority_threshold: 3.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 3, 3)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 3, 3),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 3)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 3, 3),
                majority_threshold: 3.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 2)).0,
            PolicyEvaluation::Nay {
                vote_counts: VoteCounts::new(8, 1, 3, 2),
                majority_threshold: 2.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 4)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 3, 4),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 3, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 4, 3, 0),
                majority_threshold: 3.5,
            },
        );
    }

    #[test]
    fn test_medium_spender() {
        let policy = Policy::policy_for_track(&Track::MediumSpender);
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 0, 0)).0,
            PolicyEvaluation::ParticipationNotMet {
                vote_counts: VoteCounts::new(8, 1, 0, 0),
                participation_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 1)).0,
            PolicyEvaluation::ParticipationNotMet {
                vote_counts: VoteCounts::new(8, 1, 1, 1),
                participation_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 1, 2),
                majority_threshold: 2.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 1, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 1, 2),
                majority_threshold: 2.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 3, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 3, 2),
                majority_threshold: 3.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 4, 2)).0,
            PolicyEvaluation::Nay {
                vote_counts: VoteCounts::new(8, 2, 4, 2),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 4)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 3, 4),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 3, 1)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 4, 3, 1),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 2, 1)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 4, 2, 1),
                majority_threshold: 3.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 5, 3, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 5, 3, 0),
                majority_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 3, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 4, 3, 0),
                majority_threshold: 3.5,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 3, 1)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 4, 3, 1),
                majority_threshold: 4.0,
            },
        );
    }

    #[test]
    fn test_big_spender() {
        let policy = Policy::policy_for_track(&Track::BigSpender);
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 0, 0)).0,
            PolicyEvaluation::ParticipationNotMet {
                vote_counts: VoteCounts::new(8, 1, 0, 0),
                participation_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 1)).0,
            PolicyEvaluation::ParticipationNotMet {
                vote_counts: VoteCounts::new(8, 1, 1, 1),
                participation_threshold: 4.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 1, 2),
                majority_threshold: 2.4,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 3, 1, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 3, 1, 0),
                majority_threshold: 2.4,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 1, 1)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 1, 1),
                majority_threshold: 2.4,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 1, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 1, 2),
                majority_threshold: 3.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 2, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 2, 2),
                majority_threshold: 3.6,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 3, 1)).0,
            PolicyEvaluation::Nay {
                vote_counts: VoteCounts::new(8, 2, 3, 1),
                majority_threshold: 3.6,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 3, 2)).0,
            PolicyEvaluation::Nay {
                vote_counts: VoteCounts::new(8, 2, 3, 2),
                majority_threshold: 4.2,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 4, 2)).0,
            PolicyEvaluation::Nay {
                vote_counts: VoteCounts::new(8, 2, 4, 2),
                majority_threshold: 4.8,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 2, 1, 3)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 2, 1, 3),
                majority_threshold: 3.6,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 1, 4)).0,
            PolicyEvaluation::MajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 1, 4),
                majority_threshold: 3.0,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 4)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 3, 4),
                majority_threshold: 4.8,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 0, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 4, 0, 0),
                majority_threshold: 2.4,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 1, 3, 4)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 1, 3, 4),
                majority_threshold: 4.8,
            },
        );
    }

    #[test]
    fn test_root() {
        let policy = Policy::policy_for_track(&Track::Root);
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 5, 3, 0)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 5, 3, 0),
                majority_threshold: 4.8,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 5, 2, 1)).0,
            PolicyEvaluation::Aye {
                vote_counts: VoteCounts::new(8, 5, 2, 1),
                majority_threshold: 4.8,
            },
        );
        assert_eq!(
            policy.evaluate(VoteCounts::new(8, 4, 2, 2)).0,
            PolicyEvaluation::AyeAbstainMajorityAbstain {
                vote_counts: VoteCounts::new(8, 4, 2, 2),
                majority_threshold: 4.8,
            },
        );
    }
}
