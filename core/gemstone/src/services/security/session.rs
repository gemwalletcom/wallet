use super::model::{GemAuthPromptOutcome, GemLockPeriod};
use super::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemAppLockSettings {
    pub authentication_required: bool,
    pub privacy_lock_enabled: bool,
    pub lock_period: GemLockPeriod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAppLockPhase {
    Locked,
    Unlocking { attempt: u32, interrupted: bool },
    UnlockCancelled,
    Unlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAppLockScreen {
    Hidden,
    Cover,
    Lock { unlock_button: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemAppLockViewState {
    pub screen: GemAppLockScreen,
    pub is_unlocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemAppLockSession {
    pub settings: GemAppLockSettings,
    pub phase: GemAppLockPhase,
    pub attempts: u32,
    pub is_covered: bool,
    pub has_unlocked: bool,
    pub backgrounded_at_milliseconds: Option<i64>,
}

#[uniffi::export]
pub fn new_app_lock_session(settings: GemAppLockSettings) -> GemAppLockSession {
    let phase = if settings.authentication_required {
        GemAppLockPhase::Locked
    } else {
        GemAppLockPhase::Unlocked
    };
    GemAppLockSession {
        settings,
        phase,
        attempts: 0,
        is_covered: false,
        has_unlocked: !settings.authentication_required,
        backgrounded_at_milliseconds: None,
    }
}

#[uniffi::export]
impl GemAppLockSession {
    pub fn on_settings_changed(&self, settings: GemAppLockSettings) -> Self {
        if settings.authentication_required {
            return Self { settings, ..*self };
        }
        Self {
            settings,
            phase: GemAppLockPhase::Unlocked,
            is_covered: false,
            has_unlocked: true,
            backgrounded_at_milliseconds: None,
            ..*self
        }
    }

    pub fn on_active(&self, now_milliseconds: i64, has_pending_request: bool) -> Self {
        let phase = match self.phase {
            GemAppLockPhase::Unlocked if self.relock_due(now_milliseconds, has_pending_request) => GemAppLockPhase::Locked,
            GemAppLockPhase::Unlocking { interrupted: true, .. } => GemAppLockPhase::Locked,
            phase => phase,
        };
        let session = Self {
            phase,
            is_covered: false,
            ..*self
        };
        match session.phase {
            GemAppLockPhase::Locked => session.next_attempt(),
            _ => session,
        }
    }

    pub fn on_inactive(&self, own_prompt_visible: bool) -> Self {
        Self {
            is_covered: !own_prompt_visible,
            ..*self
        }
    }

    pub fn on_background(&self, now_milliseconds: i64) -> Self {
        let (phase, backgrounded_at_milliseconds) = match self.phase {
            GemAppLockPhase::Unlocked => (self.phase, Some(now_milliseconds)),
            GemAppLockPhase::Unlocking { attempt, .. } => (GemAppLockPhase::Unlocking { attempt, interrupted: true }, self.backgrounded_at_milliseconds),
            phase => (phase, self.backgrounded_at_milliseconds),
        };
        Self {
            phase,
            is_covered: true,
            backgrounded_at_milliseconds,
            ..*self
        }
    }

    pub fn on_unlock_requested(&self) -> Self {
        match self.phase {
            GemAppLockPhase::UnlockCancelled => self.next_attempt(),
            _ => *self,
        }
    }

    pub fn on_unlocked(&self, attempt: u32) -> Self {
        if !self.is_unlocking(attempt) {
            return *self;
        }
        Self {
            phase: GemAppLockPhase::Unlocked,
            is_covered: false,
            has_unlocked: true,
            backgrounded_at_milliseconds: None,
            ..*self
        }
    }

    pub fn on_unlock_failed(&self, attempt: u32, outcome: GemAuthPromptOutcome) -> Self {
        let GemAppLockPhase::Unlocking { interrupted, .. } = self.phase else {
            return *self;
        };
        if !self.is_unlocking(attempt) {
            return *self;
        }
        let phase = if interrupted && outcome == GemAuthPromptOutcome::CancelledBySystem {
            GemAppLockPhase::Locked
        } else {
            GemAppLockPhase::UnlockCancelled
        };
        Self { phase, ..*self }
    }

    pub fn view_state(&self, now_milliseconds: i64) -> GemAppLockViewState {
        GemAppLockViewState {
            screen: self.screen(now_milliseconds),
            is_unlocked: self.phase == GemAppLockPhase::Unlocked,
        }
    }
}

impl GemAppLockSession {
    fn screen(&self, now_milliseconds: i64) -> GemAppLockScreen {
        match self.phase {
            GemAppLockPhase::Locked | GemAppLockPhase::Unlocking { .. } => GemAppLockScreen::Lock { unlock_button: false },
            GemAppLockPhase::UnlockCancelled => GemAppLockScreen::Lock { unlock_button: true },
            GemAppLockPhase::Unlocked if self.covers_content(now_milliseconds) => GemAppLockScreen::Cover,
            GemAppLockPhase::Unlocked => GemAppLockScreen::Hidden,
        }
    }

    fn covers_content(&self, now_milliseconds: i64) -> bool {
        self.settings.authentication_required && self.is_covered && (self.settings.privacy_lock_enabled || self.relock_due(now_milliseconds, false))
    }

    fn relock_due(&self, now_milliseconds: i64, has_pending_request: bool) -> bool {
        let Some(backgrounded_at_milliseconds) = self.backgrounded_at_milliseconds else {
            return false;
        };
        rules::should_relock(
            now_milliseconds - backgrounded_at_milliseconds,
            self.settings.lock_period.minutes(),
            self.settings.authentication_required,
            has_pending_request,
        )
    }

    fn is_unlocking(&self, attempt: u32) -> bool {
        matches!(self.phase, GemAppLockPhase::Unlocking { attempt: running, .. } if running == attempt)
    }

    fn next_attempt(self) -> Self {
        let attempts = self.attempts + 1;
        Self {
            phase: GemAppLockPhase::Unlocking {
                attempt: attempts,
                interrupted: false,
            },
            attempts,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: i64 = 60_000;

    fn settings(privacy_lock_enabled: bool) -> GemAppLockSettings {
        GemAppLockSettings {
            authentication_required: true,
            privacy_lock_enabled,
            lock_period: GemLockPeriod::OneMinute,
        }
    }

    fn unlocked(privacy_lock_enabled: bool) -> GemAppLockSession {
        new_app_lock_session(settings(privacy_lock_enabled)).on_active(0, false).on_unlocked(1)
    }

    #[test]
    fn test_activation_starts_one_unlock_attempt_and_success_unlocks() {
        let session = new_app_lock_session(settings(false));
        assert_eq!(session.phase, GemAppLockPhase::Locked);
        assert_eq!(session.view_state(0).screen, GemAppLockScreen::Lock { unlock_button: false });

        let unlocking = session.on_active(0, false);
        assert_eq!(unlocking.phase, GemAppLockPhase::Unlocking { attempt: 1, interrupted: false });
        assert_eq!(unlocking.on_active(1, false).attempts, 1, "a second activation joins the running attempt");

        let unlocked = unlocking.on_unlocked(1);
        assert_eq!(
            unlocked.view_state(0),
            GemAppLockViewState {
                screen: GemAppLockScreen::Hidden,
                is_unlocked: true
            }
        );
        assert!(unlocked.has_unlocked);
    }

    #[test]
    fn test_authentication_off_never_locks_or_covers() {
        let session = new_app_lock_session(GemAppLockSettings {
            authentication_required: false,
            ..settings(true)
        });
        assert!(session.has_unlocked);
        let backgrounded = session.on_background(0).on_active(MINUTE * 10, false);
        assert_eq!(
            backgrounded.view_state(MINUTE * 10),
            GemAppLockViewState {
                screen: GemAppLockScreen::Hidden,
                is_unlocked: true
            }
        );
        assert_eq!(session.on_background(0).view_state(0).screen, GemAppLockScreen::Hidden);
    }

    #[test]
    fn test_own_prompt_keeps_the_screen_visible_but_leaving_hides_it() {
        let session = unlocked(true);
        assert_eq!(session.on_inactive(true).view_state(0).screen, GemAppLockScreen::Hidden);
        assert_eq!(session.on_inactive(false).view_state(0).screen, GemAppLockScreen::Cover);
        assert_eq!(session.on_inactive(true).on_background(0).view_state(0).screen, GemAppLockScreen::Cover);
        assert_eq!(session.on_background(0).on_active(1, false).view_state(1).screen, GemAppLockScreen::Hidden);
    }

    #[test]
    fn test_without_privacy_lock_the_cover_appears_only_when_a_relock_is_due() {
        let session = unlocked(false).on_background(0);
        assert_eq!(session.view_state(MINUTE).screen, GemAppLockScreen::Hidden);
        assert_eq!(session.view_state(MINUTE + 1).screen, GemAppLockScreen::Cover);
    }

    #[test]
    fn test_relock_measures_the_latest_background_not_the_first() {
        let session = unlocked(false).on_background(0).on_active(30_000, false);
        assert_eq!(session.phase, GemAppLockPhase::Unlocked);

        let short_leave = session.on_background(MINUTE * 5).on_active(MINUTE * 5 + 1_000, false);
        assert_eq!(short_leave.phase, GemAppLockPhase::Unlocked, "a long active stretch must not count as time away");

        let long_leave = session.on_background(MINUTE * 5).on_active(MINUTE * 6 + 1, false);
        assert_eq!(long_leave.phase, GemAppLockPhase::Unlocking { attempt: 2, interrupted: false });
        assert!(long_leave.has_unlocked, "the wallet stays mounted under the lock screen");
    }

    #[test]
    fn test_a_pending_request_holds_the_relock_off() {
        let session = unlocked(false).on_background(0);
        assert_eq!(session.on_active(MINUTE * 10, true).phase, GemAppLockPhase::Unlocked);
        assert_eq!(session.on_active(MINUTE * 10, false).phase, GemAppLockPhase::Unlocking { attempt: 2, interrupted: false });
    }

    #[test]
    fn test_cancelled_unlock_waits_for_the_button() {
        let session = new_app_lock_session(settings(false)).on_active(0, false);
        let cancelled = session.on_unlock_failed(1, GemAuthPromptOutcome::CancelledByUser);
        assert_eq!(cancelled.view_state(0).screen, GemAppLockScreen::Lock { unlock_button: true });
        assert_eq!(
            cancelled.on_active(1, false).phase,
            GemAppLockPhase::UnlockCancelled,
            "activation does not retry after a cancel"
        );
        assert_eq!(cancelled.on_unlock_requested().phase, GemAppLockPhase::Unlocking { attempt: 2, interrupted: false });
        assert_eq!(session.on_unlock_failed(1, GemAuthPromptOutcome::CancelledBySystem).phase, GemAppLockPhase::UnlockCancelled);
        assert_eq!(session.on_unlock_failed(1, GemAuthPromptOutcome::LockedOut).phase, GemAppLockPhase::UnlockCancelled);
    }

    #[test]
    fn test_backgrounding_interrupts_the_attempt_and_activation_reprompts() {
        let interrupted = new_app_lock_session(settings(false)).on_active(0, false).on_background(0);
        assert_eq!(interrupted.phase, GemAppLockPhase::Unlocking { attempt: 1, interrupted: true });

        let relocked = interrupted.on_unlock_failed(1, GemAuthPromptOutcome::CancelledBySystem);
        assert_eq!(relocked.phase, GemAppLockPhase::Locked);
        assert_eq!(relocked.on_active(1, false).phase, GemAppLockPhase::Unlocking { attempt: 2, interrupted: false });

        let replaced = interrupted.on_active(1, false);
        assert_eq!(replaced.phase, GemAppLockPhase::Unlocking { attempt: 2, interrupted: false });
        assert_eq!(
            replaced.on_unlock_failed(1, GemAuthPromptOutcome::CancelledBySystem),
            replaced,
            "a stale outcome is ignored"
        );
        assert_eq!(replaced.on_unlocked(1), replaced);
        assert_eq!(replaced.on_unlocked(2).phase, GemAppLockPhase::Unlocked);
    }

    #[test]
    fn test_turning_authentication_off_unlocks_and_on_keeps_the_current_phase() {
        let covered = unlocked(true).on_background(0);
        let off = covered.on_settings_changed(GemAppLockSettings {
            authentication_required: false,
            ..settings(true)
        });
        assert_eq!(
            off.view_state(MINUTE * 10),
            GemAppLockViewState {
                screen: GemAppLockScreen::Hidden,
                is_unlocked: true
            }
        );

        let on = off.on_settings_changed(settings(true));
        assert_eq!(
            on.phase,
            GemAppLockPhase::Unlocked,
            "enabling authentication does not lock the screen the user is looking at"
        );
        assert_eq!(
            on.on_background(0).on_active(MINUTE + 1, false).phase,
            GemAppLockPhase::Unlocking { attempt: 2, interrupted: false }
        );
    }
}
