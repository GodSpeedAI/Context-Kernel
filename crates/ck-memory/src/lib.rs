#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventRetentionClass {
    Pinnable,
    Ephemeral,
    Permanent,
}

pub fn classify_event_kind(kind: &str) -> EventRetentionClass {
    match kind {
        "decision" | "architecture" | "requirement" | "constraint" => EventRetentionClass::Pinnable,
        "pinned" | "milestone" => EventRetentionClass::Permanent,
        _ => EventRetentionClass::Ephemeral,
    }
}

#[derive(Debug, Clone)]
pub struct SessionRetentionPolicy {
    pub preserve_pinned: bool,
    pub preserve_pinnable_kinds: bool,
}

impl Default for SessionRetentionPolicy {
    fn default() -> Self {
        Self {
            preserve_pinned: true,
            preserve_pinnable_kinds: false,
        }
    }
}

impl SessionRetentionPolicy {
    pub fn should_preserve_event(&self, pinned: bool, _created_at: &str) -> bool {
        if pinned && self.preserve_pinned {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryRecallScope {
    CurrentSession,
    RecentSessions,
    AllSessions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_policy_preserves_pinned_events() {
        let policy = SessionRetentionPolicy::default();
        assert!(policy.should_preserve_event(true, "2026-01-01T00:00:00Z"));
    }

    #[test]
    fn retention_policy_does_not_preserve_unpinned_by_default() {
        let policy = SessionRetentionPolicy::default();
        assert!(!policy.should_preserve_event(false, "2026-01-01T00:00:00Z"));
    }

    #[test]
    fn event_kind_classifies_decision_as_pinnable() {
        assert_eq!(
            classify_event_kind("decision"),
            EventRetentionClass::Pinnable
        );
    }

    #[test]
    fn event_kind_classifies_architecture_as_pinnable() {
        assert_eq!(
            classify_event_kind("architecture"),
            EventRetentionClass::Pinnable
        );
    }

    #[test]
    fn event_kind_classifies_note_as_ephemeral() {
        assert_eq!(classify_event_kind("note"), EventRetentionClass::Ephemeral);
    }

    #[test]
    fn event_kind_classifies_pinned_as_permanent() {
        assert_eq!(
            classify_event_kind("pinned"),
            EventRetentionClass::Permanent
        );
    }
}
