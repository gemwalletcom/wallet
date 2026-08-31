#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemLockPeriod {
    Immediate,
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    SixHours,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemAuthPromptOutcome {
    CancelledByUser,
    CancelledBySystem,
    Unavailable,
    LockedOut,
    Transient,
    Failed,
}
