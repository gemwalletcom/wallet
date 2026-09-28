const COMPLETE: &str = "IC_COMPLETE";
const ERROR: &str = "IC_ERROR";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationOutcome {
    Complete,
    Error,
    Ignored,
}

pub fn verification_outcome(message_type: &str) -> VerificationOutcome {
    match message_type {
        COMPLETE => VerificationOutcome::Complete,
        ERROR => VerificationOutcome::Error,
        _ => VerificationOutcome::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_form_reports_completion_and_failure() {
        assert_eq!(verification_outcome("IC_COMPLETE"), VerificationOutcome::Complete);
        assert_eq!(verification_outcome("IC_ERROR"), VerificationOutcome::Error);
        assert_eq!(verification_outcome("IC_PROGRESS"), VerificationOutcome::Ignored);
        assert_eq!(verification_outcome(""), VerificationOutcome::Ignored);
    }
}
