use crate::solver::{OllAlgorithm, PllAlgorithm, SkillId};
use std::collections::{HashMap, HashSet};

const STARTING_TPS: f32 = 1.0;
const STARTING_RUN_MAX_TPS: f32 = 1.5;
const STARTING_TPS_CAP: f32 = 6.0;
const STARTING_RECOGNITION_TIME: f32 = 1.0;
const STARTING_RECOGNITION_FLOOR: f32 = 0.25;
const TURNING_MASTERY_SCALE: f32 = 20.0;
const RECOGNITION_MASTERY_SCALE: f32 = 20.0;
const INITIAL_LEVEL_XP: f32 = 20.0;
const LEVEL_XP_MULTIPLIER: f32 = 1.25;
const LEVEL_TPS_MULTIPLIER: f32 = 1.08;
const PRESTIGE_XP_SCALE: f32 = 100.0;
const KNOWLEDGE_PER_PRESTIGE_BRACKET: u32 = 10;
const PRESTIGE_XP_MULTIPLIER: f32 = 1.08;
const PRESTIGE_TPS_CAP_MULTIPLIER: f32 = 1.1;
const PRESTIGE_RECOGNITION_FLOOR_MULTIPLIER: f32 = 0.9;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PrestigeTechnique {
    TwoLookPll,
    TwoLookOll,
}

impl PrestigeTechnique {
    pub fn cost(self) -> u32 {
        match self {
            Self::TwoLookPll => 5,
            Self::TwoLookOll => 25,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Technique {
    DirectCross,
    IntuitiveF2L,
    Oll(OllAlgorithm),
    Pll(PllAlgorithm),
}

impl Technique {
    pub fn cost(self) -> u32 {
        1
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Proficiency {
    pub recognition_xp: f32,
    pub turning_xp: f32,
}

pub struct PlayerSkills {
    max_tps: f32,
    max_tps_cap: f32,
    recognition_floor: f32,
    xp: f32,
    run_xp: f32,
    level: u32,
    skill_points: u32,
    cube_knowledge: u32,
    prestige_count: u32,
    prestige_techniques: HashSet<PrestigeTechnique>,
    tps_cap_upgrades: u32,
    recognition_floor_upgrades: u32,
    unlocked_techniques: HashSet<Technique>,
    proficiency: HashMap<SkillId, Proficiency>,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        Self {
            max_tps: STARTING_RUN_MAX_TPS,
            max_tps_cap: STARTING_TPS_CAP,
            recognition_floor: STARTING_RECOGNITION_FLOOR,
            xp: 0.0,
            run_xp: 0.0,
            level: 1,
            skill_points: 0,
            cube_knowledge: 0,
            prestige_count: 0,
            prestige_techniques: HashSet::new(),
            tps_cap_upgrades: 0,
            recognition_floor_upgrades: 0,
            unlocked_techniques: HashSet::new(),
            proficiency: HashMap::new(),
        }
    }
}

impl PlayerSkills {
    pub fn max_tps(&self) -> f32 {
        self.max_tps
    }

    pub fn xp(&self) -> f32 {
        self.xp
    }

    pub fn run_xp(&self) -> f32 {
        self.run_xp
    }

    pub fn max_tps_cap(&self) -> f32 {
        self.max_tps_cap
    }

    pub fn recognition_floor(&self) -> f32 {
        self.recognition_floor
    }

    pub fn cube_knowledge(&self) -> u32 {
        self.cube_knowledge
    }

    pub fn xp_multiplier(&self) -> f32 {
        PRESTIGE_XP_MULTIPLIER.powi(self.cube_knowledge as i32)
    }

    pub fn prestige_reward(&self) -> u32 {
        ((self.run_xp / PRESTIGE_XP_SCALE).sqrt().floor() as u32)
            .saturating_mul(KNOWLEDGE_PER_PRESTIGE_BRACKET)
    }

    pub fn next_prestige_threshold(&self) -> f32 {
        let next_bracket = self.prestige_reward() / KNOWLEDGE_PER_PRESTIGE_BRACKET + 1;
        PRESTIGE_XP_SCALE * (next_bracket as f32).powi(2)
    }

    pub fn has_prestiged(&self) -> bool {
        self.prestige_count > 0
    }

    pub fn has_access(&self, technique: PrestigeTechnique) -> bool {
        self.prestige_techniques.contains(&technique)
    }

    pub fn can_buy_access(&self, technique: PrestigeTechnique) -> bool {
        self.has_prestiged()
            && !self.has_access(technique)
            && self.cube_knowledge >= technique.cost()
            && (technique == PrestigeTechnique::TwoLookPll
                || self.has_access(PrestigeTechnique::TwoLookPll))
    }

    pub fn buy_access(&mut self, technique: PrestigeTechnique) -> bool {
        if !self.can_buy_access(technique) {
            return false;
        }
        self.cube_knowledge -= technique.cost();
        self.prestige_techniques.insert(technique);
        true
    }

    pub fn tps_cap_upgrade_cost(&self) -> u32 {
        2 * (self.tps_cap_upgrades + 1)
    }

    pub fn recognition_floor_upgrade_cost(&self) -> u32 {
        2 * (self.recognition_floor_upgrades + 1)
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn skill_points(&self) -> u32 {
        self.skill_points
    }

    pub fn xp_to_next_level(&self) -> f32 {
        INITIAL_LEVEL_XP * LEVEL_XP_MULTIPLIER.powi((self.level - 1) as i32)
    }

    pub fn xp_progress(&self) -> f32 {
        self.xp / self.xp_to_next_level()
    }

    pub fn award_solve_xp(&mut self, solve_seconds: f32) {
        let earned = solve_xp(solve_seconds) * self.xp_multiplier();
        self.run_xp += earned;
        self.award_xp(earned);
    }

    fn award_xp(&mut self, amount: f32) {
        self.xp += amount;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
            self.skill_points += 1;
            self.max_tps = (self.max_tps * LEVEL_TPS_MULTIPLIER).min(self.max_tps_cap);
        }
    }

    pub fn prestige(&mut self) -> bool {
        let reward = self.prestige_reward();
        if reward == 0 {
            return false;
        }

        self.cube_knowledge += reward;
        self.prestige_count += 1;
        self.max_tps = STARTING_RUN_MAX_TPS.min(self.max_tps_cap);
        self.xp = 0.0;
        self.run_xp = 0.0;
        self.level = 1;
        self.skill_points = 0;
        self.unlocked_techniques.clear();
        self.proficiency.clear();
        true
    }

    pub fn buy_tps_cap_upgrade(&mut self) -> bool {
        let cost = self.tps_cap_upgrade_cost();
        if self.cube_knowledge < cost {
            return false;
        }
        self.cube_knowledge -= cost;
        self.tps_cap_upgrades += 1;
        self.max_tps_cap *= PRESTIGE_TPS_CAP_MULTIPLIER;
        true
    }

    pub fn buy_recognition_floor_upgrade(&mut self) -> bool {
        let cost = self.recognition_floor_upgrade_cost();
        if self.cube_knowledge < cost {
            return false;
        }
        self.cube_knowledge -= cost;
        self.recognition_floor_upgrades += 1;
        self.recognition_floor *= PRESTIGE_RECOGNITION_FLOOR_MULTIPLIER;
        true
    }

    pub fn is_unlocked(&self, technique: Technique) -> bool {
        self.unlocked_techniques.contains(&technique)
    }

    pub fn unlock(&mut self, technique: Technique) -> bool {
        if self.is_unlocked(technique) || self.skill_points < technique.cost() {
            return false;
        }
        if technique != Technique::DirectCross && !self.is_unlocked(Technique::DirectCross) {
            return false;
        }
        match technique {
            Technique::Pll(_) if !self.has_access(PrestigeTechnique::TwoLookPll) => return false,
            Technique::Oll(_) if !self.has_access(PrestigeTechnique::TwoLookOll) => return false,
            _ => {}
        }

        self.skill_points -= technique.cost();
        self.unlocked_techniques.insert(technique);
        true
    }

    pub fn recognition_time(&self, skill: SkillId) -> f32 {
        let xp = self.proficiency(skill).recognition_xp;
        self.recognition_floor
            + (STARTING_RECOGNITION_TIME - self.recognition_floor)
                * (-xp / RECOGNITION_MASTERY_SCALE).exp()
    }

    pub fn turning_tps(&self, skill: SkillId) -> f32 {
        let xp = self.proficiency(skill).turning_xp;
        let mastery = 1.0 - (-xp / TURNING_MASTERY_SCALE).exp();
        STARTING_TPS + mastery * (self.max_tps - STARTING_TPS)
    }

    pub fn practice_recognition(&mut self, skill: SkillId) {
        self.proficiency.entry(skill).or_default().recognition_xp += 1.0;
    }

    pub fn practice_turning(&mut self, skill: SkillId) {
        self.proficiency.entry(skill).or_default().turning_xp += 1.0;
    }

    pub fn proficiency(&self, skill: SkillId) -> Proficiency {
        self.proficiency.get(&skill).copied().unwrap_or_default()
    }
}

pub fn solve_xp(solve_seconds: f32) -> f32 {
    if solve_seconds >= 120.0 {
        10.0
    } else if solve_seconds >= 60.0 {
        10.0 + (120.0 - solve_seconds) / 6.0
    } else {
        let world_record_progress = ((60.0 - solve_seconds) / 57.0).clamp(0.0, 1.0);
        20.0 + 80.0 * world_record_progress.powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_improves_only_the_practiced_skill() {
        let mut skills = PlayerSkills {
            max_tps: STARTING_TPS * 1.01,
            ..Default::default()
        };
        let practiced = SkillId::BuildDaisy;
        let untouched = SkillId::YellowCross;
        let initial_recognition = skills.recognition_time(practiced);
        let initial_tps = skills.turning_tps(practiced);

        skills.practice_recognition(practiced);
        skills.practice_turning(practiced);

        assert!(skills.recognition_time(practiced) < initial_recognition);
        assert!(skills.turning_tps(practiced) > initial_tps);
        assert_eq!(skills.recognition_time(untouched), initial_recognition);
        assert_eq!(skills.turning_tps(untouched), initial_tps);
    }

    #[test]
    fn proficiency_approaches_but_does_not_exceed_its_limits() {
        let mut skills = PlayerSkills::default();
        let skill = SkillId::BuildDaisy;
        for _ in 0..1_000 {
            skills.practice_recognition(skill);
            skills.practice_turning(skill);
        }

        assert!(skills.recognition_time(skill) >= skills.recognition_floor());
        assert!(skills.turning_tps(skill) <= skills.max_tps());
    }

    #[test]
    fn leveling_grants_a_skill_point_and_tps_increase() {
        let mut skills = PlayerSkills::default();
        let starting_tps = skills.max_tps();

        let required_xp = skills.xp_to_next_level();
        skills.award_xp(required_xp / 2.0);
        assert_eq!(skills.level(), 1);
        assert_eq!(skills.xp_progress(), 0.5);
        skills.award_xp(required_xp / 2.0);

        assert_eq!(skills.level(), 2);
        assert_eq!(skills.skill_points(), 1);
        assert_eq!(skills.xp(), 0.0);
        assert!((skills.max_tps() - starting_tps * LEVEL_TPS_MULTIPLIER).abs() < 1e-6);
    }

    #[test]
    fn skill_points_unlock_techniques() {
        let mut skills = PlayerSkills::default();

        assert!(!skills.unlock(Technique::DirectCross));
        skills.skill_points = 1;
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.is_unlocked(Technique::DirectCross));
        assert_eq!(skills.skill_points(), 0);
        assert!(!skills.unlock(Technique::DirectCross));
    }

    #[test]
    fn intuitive_f2l_requires_direct_cross() {
        let mut skills = PlayerSkills {
            skill_points: 2,
            ..Default::default()
        };

        assert_eq!(skills.skill_points(), 2);
        assert!(!skills.unlock(Technique::IntuitiveF2L));
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.unlock(Technique::IntuitiveF2L));
    }

    #[test]
    fn pll_algorithms_branch_directly_from_cross() {
        let mut skills = PlayerSkills {
            skill_points: 2,
            prestige_techniques: HashSet::from([PrestigeTechnique::TwoLookPll]),
            ..Default::default()
        };

        assert!(!skills.unlock(Technique::Pll(PllAlgorithm::Ua)));
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.unlock(Technique::Pll(PllAlgorithm::Ua)));
        assert!(!skills.is_unlocked(Technique::IntuitiveF2L));
    }

    #[test]
    fn oll_algorithms_branch_directly_from_cross() {
        let mut skills = PlayerSkills {
            skill_points: 2,
            prestige_techniques: HashSet::from([PrestigeTechnique::TwoLookOll]),
            ..Default::default()
        };

        assert!(!skills.unlock(Technique::Oll(OllAlgorithm::Sune)));
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.unlock(Technique::Oll(OllAlgorithm::Sune)));
        assert!(!skills.is_unlocked(Technique::IntuitiveF2L));
    }

    #[test]
    fn solve_xp_scales_from_two_minutes_to_world_record_pace() {
        assert_eq!(solve_xp(180.0), 10.0);
        assert_eq!(solve_xp(120.0), 10.0);
        assert_eq!(solve_xp(60.0), 20.0);
        assert_eq!(solve_xp(3.0), 100.0);
        assert!(solve_xp(30.0) > solve_xp(45.0));
        assert!(solve_xp(30.0) < 100.0);
    }

    #[test]
    fn one_hundred_run_xp_earns_the_first_cube_knowledge() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            level: 8,
            skill_points: 3,
            ..Default::default()
        };
        skills.unlocked_techniques.insert(Technique::DirectCross);

        assert_eq!(skills.prestige_reward(), 10);
        assert!(skills.prestige());
        assert_eq!(skills.cube_knowledge(), 10);
        assert_eq!(skills.run_xp(), 0.0);
        assert_eq!(skills.level(), 1);
        assert_eq!(skills.skill_points(), 0);
        assert!(!skills.is_unlocked(Technique::DirectCross));
    }

    #[test]
    fn prestige_upgrades_trade_the_banked_xp_bonus_for_permanent_caps() {
        let mut skills = PlayerSkills {
            cube_knowledge: 4,
            prestige_count: 1,
            prestige_techniques: HashSet::from([PrestigeTechnique::TwoLookPll]),
            ..Default::default()
        };
        let initial_cap = skills.max_tps_cap();
        let initial_floor = skills.recognition_floor();
        assert!((skills.xp_multiplier() - 1.08_f32.powi(4)).abs() < 1e-6);

        assert!(skills.buy_tps_cap_upgrade());
        assert!(skills.max_tps_cap() > initial_cap);
        assert!(skills.buy_recognition_floor_upgrade());
        assert!(skills.recognition_floor() < initial_floor);
        assert_eq!(skills.cube_knowledge(), 0);
        assert_eq!(skills.xp_multiplier(), 1.0);
    }

    #[test]
    fn prestige_rewards_use_ten_knowledge_brackets() {
        let mut skills = PlayerSkills::default();
        for (xp, reward) in [
            (99.0, 0),
            (100.0, 10),
            (399.0, 10),
            (400.0, 20),
            (900.0, 30),
        ] {
            skills.run_xp = xp;
            assert_eq!(skills.prestige_reward(), reward);
            assert!(skills.next_prestige_threshold() > xp);
        }
        skills.run_xp = 100.0;
        assert_eq!(skills.next_prestige_threshold(), 400.0);
        skills.run_xp = 400.0;
        assert_eq!(skills.next_prestige_threshold(), 900.0);
    }

    #[test]
    fn optional_pll_purchase_leaves_five_knowledge() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            ..Default::default()
        };
        assert!(!skills.can_buy_access(PrestigeTechnique::TwoLookPll));
        assert!(skills.prestige());
        assert!(skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert_eq!(skills.cube_knowledge(), 5);
        assert!((skills.xp_multiplier() - 1.469328).abs() < 1e-5);
        assert!(!skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert_eq!(skills.cube_knowledge(), 5);
    }

    #[test]
    fn spending_below_pll_cost_does_not_prevent_earning_it_later() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            ..Default::default()
        };
        assert!(skills.prestige());
        assert!(skills.buy_tps_cap_upgrade());
        assert!(skills.buy_recognition_floor_upgrade());
        assert!(skills.buy_tps_cap_upgrade());
        assert_eq!(skills.cube_knowledge(), 2);
        assert!(!skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert!(!skills.has_access(PrestigeTechnique::TwoLookPll));
        for _ in 0..10 {
            skills.award_solve_xp(120.0);
        }
        assert!(skills.prestige());
        assert_eq!(skills.cube_knowledge(), 12);
        assert!(skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert_eq!(skills.cube_knowledge(), 7);
    }

    #[test]
    fn first_run_cannot_learn_last_layer_algorithms() {
        let mut skills = PlayerSkills {
            skill_points: 10,
            ..Default::default()
        };
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.unlock(Technique::IntuitiveF2L));
        for algorithm in PllAlgorithm::ALL {
            assert!(!skills.unlock(Technique::Pll(algorithm)));
        }
        for algorithm in OllAlgorithm::ALL {
            assert!(!skills.unlock(Technique::Oll(algorithm)));
        }
        assert_eq!(skills.skill_points(), 8);
    }

    #[test]
    fn oll_access_rewards_a_longer_second_run() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            ..Default::default()
        };
        assert!(skills.prestige());
        assert!(skills.buy_access(PrestigeTechnique::TwoLookPll));
        skills.run_xp = 100.0;
        assert_eq!(skills.cube_knowledge() + skills.prestige_reward(), 15);
        assert!(!skills.buy_access(PrestigeTechnique::TwoLookOll));
        skills.run_xp = 400.0;
        assert!(skills.prestige());
        assert_eq!(skills.cube_knowledge(), 25);
        assert!(skills.buy_access(PrestigeTechnique::TwoLookOll));
        assert_eq!(skills.cube_knowledge(), 0);
        assert_eq!(skills.xp_multiplier(), 1.0);
    }

    #[test]
    fn access_and_buyables_survive_resets_but_learning_resets() {
        let mut skills = PlayerSkills {
            run_xp: 900.0,
            ..Default::default()
        };
        assert!(skills.prestige());
        assert!(skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert!(skills.buy_access(PrestigeTechnique::TwoLookOll));
        skills.skill_points = 3;
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.unlock(Technique::Pll(PllAlgorithm::Ua)));
        assert!(skills.unlock(Technique::Oll(OllAlgorithm::Sune)));
        skills.practice_recognition(SkillId::BuildDaisy);
        skills.run_xp = 100.0;
        assert!(skills.prestige());
        assert!(skills.buy_tps_cap_upgrade());
        assert!(skills.buy_recognition_floor_upgrade());
        let cap = skills.max_tps_cap();
        let floor = skills.recognition_floor();
        skills.run_xp = 100.0;
        assert!(skills.prestige());
        assert!(skills.has_access(PrestigeTechnique::TwoLookPll));
        assert!(skills.has_access(PrestigeTechnique::TwoLookOll));
        assert!(!skills.is_unlocked(Technique::Pll(PllAlgorithm::Ua)));
        assert!(!skills.is_unlocked(Technique::Oll(OllAlgorithm::Sune)));
        assert_eq!(skills.proficiency(SkillId::BuildDaisy).recognition_xp, 0.0);
        assert_eq!(skills.max_tps_cap(), cap);
        assert_eq!(skills.recognition_floor(), floor);
        assert_eq!(skills.tps_cap_upgrade_cost(), 4);
        assert_eq!(skills.recognition_floor_upgrade_cost(), 4);
    }

    #[test]
    fn buyable_costs_grow_and_failed_purchases_do_not_spend() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            ..Default::default()
        };
        assert!(skills.prestige());
        assert!(skills.buy_access(PrestigeTechnique::TwoLookPll));
        assert!(skills.buy_tps_cap_upgrade());
        assert_eq!(skills.cube_knowledge(), 3);
        assert_eq!(skills.tps_cap_upgrade_cost(), 4);
        assert!(!skills.buy_tps_cap_upgrade());
        assert_eq!(skills.cube_knowledge(), 3);
        assert!(skills.buy_recognition_floor_upgrade());
        assert_eq!(skills.cube_knowledge(), 1);
        assert!((skills.xp_multiplier() - 1.08).abs() < 1e-6);
        assert!(!skills.buy_recognition_floor_upgrade());
        assert_eq!(skills.cube_knowledge(), 1);
    }

    #[test]
    fn leveling_stops_at_the_current_tps_cap() {
        let mut skills = PlayerSkills::default();
        skills.award_xp(1_000_000.0);
        assert_eq!(skills.max_tps(), skills.max_tps_cap());
    }

    #[test]
    fn full_four_look_tree_reaches_the_initial_tps_cap() {
        let full_tree_level_ups = 18;
        let projected_tps = STARTING_RUN_MAX_TPS * LEVEL_TPS_MULTIPLIER.powi(full_tree_level_ups);

        assert!((projected_tps - STARTING_TPS_CAP).abs() < 0.05);
    }
}
