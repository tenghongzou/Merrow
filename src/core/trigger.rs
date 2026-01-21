use super::TriggerContext;

pub trait Trigger {
    fn should_fire(&self, ctx: &TriggerContext) -> bool;
}
