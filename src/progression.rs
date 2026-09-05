use crate::solver::SkillId;
use std::collections::{HashMap, HashSet};

const STARTING_TPS: f32 = 1.0;
const STARTING_RECOGNITION_TIME: f32 = 0.5;
const MIN_RECOGNITION_TIME: f32 = 0.05;
const TURNING_MASTERY_SCALE: f32 = 20.0;
const RECOGNITION_MASTERY_SCALE: f32 = 20.0;
const XP_PER_SOLVE: f32 = 10.0;
const INITIAL_LEVEL_XP: f32 = 20.0;
const LEVEL_XP_MULTIPLIER: f32 = 1.25;
const LEVEL_TPS_MULTIPLIER: f32 = 1.01;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Technique {
    DirectCross,
    IntuitiveF2L,
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
    xp: f32,
    level: u32,
    skill_points: u32,
    unlocked_techniques: HashSet<Technique>,
    proficiency: HashMap<SkillId, Proficiency>,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        Self {
            max_tps: 2.0,
            xp: 0.0,
            level: 1,
            skill_points: 0,
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

    pub fn award_solve_xp(&mut self) {
        self.xp += XP_PER_SOLVE;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
            self.skill_points += 1;
            self.max_tps *= LEVEL_TPS_MULTIPLIER;
        }
    }

    pub fn is_unlocked(&self, technique: Technique) -> bool {
        self.unlocked_techniques.contains(&technique)
    }

    pub fn unlock(&mut self, technique: Technique) -> bool {
        if self.is_unlocked(technique) || self.skill_points < technique.cost() {
            return false;
        }
        if technique == Technique::IntuitiveF2L && !self.is_unlocked(Technique::DirectCross) {
            return false;
        }

        self.skill_points -= technique.cost();
        self.unlocked_techniques.insert(technique);
        true
    }

    pub fn recognition_time(&self, skill: SkillId) -> f32 {
        let xp = self.proficiency(skill).recognition_xp;
        MIN_RECOGNITION_TIME
            + (STARTING_RECOGNITION_TIME - MIN_RECOGNITION_TIME)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_improves_only_the_practiced_skill() {
        let mut skills = PlayerSkills::default();
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

        assert!(skills.recognition_time(skill) >= MIN_RECOGNITION_TIME);
        assert!(skills.turning_tps(skill) <= skills.max_tps());
    }

    #[test]
    fn leveling_grants_a_skill_point_and_tps_increase() {
        let mut skills = PlayerSkills::default();
        let starting_tps = skills.max_tps();

        skills.award_solve_xp();
        assert_eq!(skills.level(), 1);
        assert_eq!(skills.xp_progress(), 0.5);
        skills.award_solve_xp();

        assert_eq!(skills.level(), 2);
        assert_eq!(skills.skill_points(), 1);
        assert_eq!(skills.xp(), 0.0);
        assert!((skills.max_tps() - starting_tps * 1.01).abs() < f32::EPSILON);
    }

    #[test]
    fn skill_points_unlock_techniques() {
        let mut skills = PlayerSkills::default();

        assert!(!skills.unlock(Technique::DirectCross));
        skills.award_solve_xp();
        skills.award_solve_xp();
        assert!(skills.unlock(Technique::DirectCross));
        assert!(skills.is_unlocked(Technique::DirectCross));
        assert_eq!(skills.skill_points(), 0);
        assert!(!skills.unlock(Technique::DirectCross));
    }
}
