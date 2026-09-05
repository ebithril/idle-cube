use crate::solver::SkillId;
use std::collections::HashMap;

const STARTING_TPS: f32 = 1.0;
const STARTING_RECOGNITION_TIME: f32 = 0.5;
const MIN_RECOGNITION_TIME: f32 = 0.05;
const TURNING_MASTERY_SCALE: f32 = 20.0;
const RECOGNITION_MASTERY_SCALE: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Proficiency {
    pub recognition_xp: f32,
    pub turning_xp: f32,
}

pub struct PlayerSkills {
    max_tps: f32,
    proficiency: HashMap<SkillId, Proficiency>,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        Self {
            max_tps: 2.0,
            proficiency: HashMap::new(),
        }
    }
}

impl PlayerSkills {
    pub fn max_tps(&self) -> f32 {
        self.max_tps
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
}
