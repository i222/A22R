#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationIntent {
    SelectItem(String),
    BeginCreate,
    CancelEdit,
    CloseWindow,
}

#[derive(Debug, Default)]
pub struct UnsavedChangesGuard {
    pending: Option<NavigationIntent>,
}

impl UnsavedChangesGuard {
    pub fn request(&mut self, intent: NavigationIntent) -> bool {
        if self.pending.is_some() {
            return false;
        }
        self.pending = Some(intent);
        true
    }

    pub fn take(&mut self) -> Option<NavigationIntent> {
        self.pending.take()
    }

    pub fn restore(&mut self, intent: NavigationIntent) {
        self.pending = Some(intent);
    }

    pub fn clear(&mut self) {
        self.pending = None;
    }

    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_keeps_only_the_first_pending_intent() {
        let mut guard = UnsavedChangesGuard::default();
        assert!(guard.request(NavigationIntent::BeginCreate));
        assert!(!guard.request(NavigationIntent::CloseWindow));
        assert_eq!(guard.take(), Some(NavigationIntent::BeginCreate));
        assert!(!guard.is_open());
    }

    #[test]
    fn guard_can_restore_an_intent_after_failed_save() {
        let mut guard = UnsavedChangesGuard::default();
        guard.request(NavigationIntent::CancelEdit);
        let intent = guard.take().expect("pending intent");
        guard.restore(intent.clone());
        assert_eq!(guard.take(), Some(intent));
    }
}
