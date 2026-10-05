use rewards::ReferralError;

pub enum RiskAssessment {
    Allowed { risk_signal_id: i32 },
    Exceeded { risk_signal_id: i32, error: ReferralError },
}
