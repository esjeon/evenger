
use super::{DeviceId};
use super::srcdev::*;
use super::destdev::*;
use std::rc::Rc;

pub struct RuleSet {
    rules: Vec<Rc<Rule>>,
}

pub struct Rule {
    srcdev_id: Option<DeviceId>,
    target: EventTarget,
    modifier_rules: Vec<Rc<ModifierRule>>,
}

pub struct ModifierRule {
    modifiers: Vec<(Option<DeviceId>, Modifier)>,
    action_rules: Vec<ActionRule>,
}

pub struct ActionRule {
    phase: ActionPhase,
    action: Action,
}

pub enum ActionPhase {
    PreAction,
    PeriAction,
    PostAction,
}

#[allow(dead_code)]
impl RuleSet {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self,
        srcdev_id: Option<DeviceId>,
        target: EventTarget,
        modifiers: &[DeviceModifier],
        phase: ActionPhase,
    ) {
        let rule = self.get_rule_or_create(&srcdev_id, target);
        let mod_rule = rule.get_modifier_rule(modifiers);
    }

    pub fn get_rule(&self, srcdev_id: &Option<DeviceId>, target: EventTarget) -> Option<Rc<Rule>> {
        (&self.rules).iter()
            .find(move |&rule: &&Rc<Rule>|
                rule.srcdev_id == *srcdev_id &&
                rule.target == target
            )
            .map(|rule: &Rc<Rule>| Rc::clone(rule))
    }

    pub fn get_rule_or_create(&mut self, srcdev_id: &Option<DeviceId>, target: EventTarget) -> Rc<Rule> {
        match self.get_rule(srcdev_id, target) {
            Some(v) => v,
            None => {
                let rule = Rc::new(Rule::new(srcdev_id.clone(), target));
                self.rules.push(rule.clone());
                rule
            }
        }
    }
}

impl Rule {
    pub fn new(srcdev_id: Option<DeviceId>, target: EventTarget) -> Self {
        Self {
            srcdev_id,
            target,
            modifier_rules: Vec::new(),
        }
    }

    pub fn get_modifier_rule(&self, modifiers: &[DeviceModifier]) -> Option<Rc<ModifierRule>> {
        self.modifier_rules.iter()
            .find(|modr: &&Rc<ModifierRule>|
                modr.modifiers == modifiers)
            .map(|modr| modr.clone())
    }

    pub fn get_modifier_rule_or_create(&mut self, modifiers: &[DeviceModifier]) -> Rc<ModifierRule> {
        match self.get_modifier_rule(modifiers) {
            Some(v) => v,
            None => {
                let mod_rule = Rc::new(ModifierRule::new(modifiers));
                self.modifier_rules.push(mod_rule.clone());
                mod_rule
            },
        }
    }

    pub fn find_modifier_rule(&self, srcdevs: &SourceDeviceSet) -> Option<Rc<ModifierRule>> {
        self.modifier_rules.iter()
            .find(|modr|
                modr.matches(srcdevs))
            .map(|modr|
                Rc::clone(modr))
    }
}

impl ModifierRule {
    pub fn new(modifiers: &[DeviceModifier]) -> Self {
        Self {
            modifiers: Vec::from(modifiers),
            action_rules: Vec::new(),
        }
    }

    pub fn matches(&self, srcdevs: &SourceDeviceSet) -> bool {
        self.modifiers.iter()
            .all(|devmod|
                srcdevs.test_modifier(devmod.clone()))
    }
}
