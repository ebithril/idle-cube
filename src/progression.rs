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
const PRESTIGE_XP_MULTIPLIER: f32 = 1.1;
const PRESTIGE_TPS_CAP_MULTIPLIER: f32 = 1.1;
const PRESTIGE_RECOGNITION_FLOOR_MULTIPLIER: f32 = 0.9;

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
    prestige_currency: u32,
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
            prestige_currency: 0,
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

    pub fn prestige_currency(&self) -> u32 {
        self.prestige_currency
    }

    pub fn xp_multiplier(&self) -> f32 {
        PRESTIGE_XP_MULTIPLIER.powi(self.prestige_currency as i32)
    }

    pub fn prestige_reward(&self) -> u32 {
        (self.run_xp / PRESTIGE_XP_SCALE).sqrt().floor() as u32
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

        self.prestige_currency += reward;
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
        if self.prestige_currency == 0 {
            return false;
        }
        self.prestige_currency -= 1;
        self.max_tps_cap *= PRESTIGE_TPS_CAP_MULTIPLIER;
        true
    }

    pub fn buy_recognition_floor_upgrade(&mut self) -> bool {
        if self.prestige_currency == 0 {
            return false;
        }
        self.prestige_currency -= 1;
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
    fn one_hundred_run_xp_earns_the_first_prestige_currency() {
        let mut skills = PlayerSkills {
            run_xp: 100.0,
            level: 8,
            skill_points: 3,
            ..Default::default()
        };
        skills.unlocked_techniques.insert(Technique::DirectCross);

        assert_eq!(skills.prestige_reward(), 1);
        assert!(skills.prestige());
        assert_eq!(skills.prestige_currency(), 1);
        assert_eq!(skills.run_xp(), 0.0);
        assert_eq!(skills.level(), 1);
        assert_eq!(skills.skill_points(), 0);
        assert!(!skills.is_unlocked(Technique::DirectCross));
    }

    #[test]
    fn prestige_upgrades_trade_the_banked_xp_bonus_for_permanent_caps() {
        let mut skills = PlayerSkills {
            prestige_currency: 2,
            ..Default::default()
        };
        let initial_cap = skills.max_tps_cap();
        let initial_floor = skills.recognition_floor();
        assert!((skills.xp_multiplier() - 1.21).abs() < f32::EPSILON);

        assert!(skills.buy_tps_cap_upgrade());
        assert!(skills.max_tps_cap() > initial_cap);
        assert!(skills.buy_recognition_floor_upgrade());
        assert!(skills.recognition_floor() < initial_floor);
        assert_eq!(skills.prestige_currency(), 0);
        assert_eq!(skills.xp_multiplier(), 1.0);
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
