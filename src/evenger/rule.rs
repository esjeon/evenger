
use super::{DeviceId};
use super::srcdev::*;
use super::destdev::*;
use std::rc::Rc;

pub struct RuleSet {
    rules: Vec<Rc<Rule>>,
}

pub struct Rule {
    device: Option<DeviceId>,
    main: EventTarget,
    modifiers: Box<[ModifierRule]>,
    actions: Vec<ActionRule>,
}

#[derive(Clone, PartialEq)]
pub struct ModifierRule {
    device: Option<DeviceId>,
    target: Modifier,
}

#[derive(Clone)]
pub struct ActionRule {
    phase: ActionRulePhase,
    action: Action,
}

#[derive(Clone)]
pub enum ActionRulePhase {
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

    pub fn insert_rule(&mut self,
        device: Option<DeviceId>,
        main: EventTarget,
        modifiers: &[ModifierRule],
        phase: ActionRulePhase,
        action: Action,
    ) {
        self.get_or_create_rule_mut(device, main, modifiers)
            .add_action(ActionRule::new(phase, action));
    }

    pub fn match_rules(&self) {
        // TODO: implement this
    }

    fn get_or_create_rule_mut(&mut self, device: Option<DeviceId>, main: EventTarget, modifiers: &[ModifierRule]) -> &mut Rule {
        let idx: Option<usize> = self.rules.iter()
            .position(|rule: &Rc<Rule>|
                rule.device == device &&
                rule.main == main &&
                &*rule.modifiers == modifiers
            );

        let idx = match idx {
            Some(v) => v,
            None => {
                let rule = Rule::new(device, main, modifiers);
                self.rules.push(Rc::new(rule));
                self.rules.len() - 1
            }
        };

        Rc::get_mut(&mut self.rules[idx]).unwrap()
    }
}

impl Rule {
    pub fn new(
        device: Option<DeviceId>,
        main: EventTarget,
        modifiers: &[ModifierRule],
    ) -> Self {
        Self {
            device,
            main,
            modifiers: Vec::from(modifiers).into_boxed_slice(),
            actions: Vec::new(),
        }
    }

    fn add_action(&mut self, action: ActionRule) {
        self.actions.push(action);
    }
}

impl ActionRule {
    pub fn new(phase: ActionRulePhase, action: Action) -> Self {
        Self {
            phase,
            action,
        }
    }
}
