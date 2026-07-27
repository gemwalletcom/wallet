use gem_tracing::{error_fields, info_with_fields};
use primitives::NodeCheckStatus;

const FIELD_WIDTH: usize = 30;
const LATENCY_WIDTH: usize = 9;
const STATUS_WIDTH: usize = 6;

#[derive(Clone, Copy)]
pub(crate) enum ResultStatus {
    Passed,
    Warning,
    Failed,
}

impl ResultStatus {
    pub(crate) fn from_counts(warnings: u32, failures: u32) -> Self {
        if failures > 0 {
            Self::Failed
        } else if warnings > 0 {
            Self::Warning
        } else {
            Self::Passed
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Warning => "WARN",
            Self::Failed => "FAIL",
        }
    }
}

impl From<&NodeCheckStatus> for ResultStatus {
    fn from(status: &NodeCheckStatus) -> Self {
        match status {
            NodeCheckStatus::Passed { .. } => Self::Passed,
            NodeCheckStatus::Warning { .. } => Self::Warning,
            NodeCheckStatus::Failed { .. } => Self::Failed,
        }
    }
}

pub(crate) struct ResultTable {
    has_latency: bool,
}

impl ResultTable {
    pub(crate) fn start(title: &str, field: &str, has_latency: bool) -> Self {
        info_with_fields!(&format!("┌─ {title}"));
        if has_latency {
            info_with_fields!(&format!("│ {:<STATUS_WIDTH$} │ {field:<FIELD_WIDTH$} │ {:<LATENCY_WIDTH$} │ result", "status", "latency"));
        } else {
            info_with_fields!(&format!("│ {:<STATUS_WIDTH$} │ {field:<FIELD_WIDTH$} │ result", "status"));
        }
        Self { has_latency }
    }

    pub(crate) fn row(&self, status: ResultStatus, field: &str, latency_ms: Option<u64>, result: &str) {
        let label = status.label();
        let line = if self.has_latency {
            let latency = latency_ms.map_or_else(|| "-".to_string(), |latency| format!("{latency} ms"));
            format!("│ {label:<STATUS_WIDTH$} │ {field:<FIELD_WIDTH$} │ {latency:<LATENCY_WIDTH$} │ {result}")
        } else {
            format!("│ {label:<STATUS_WIDTH$} │ {field:<FIELD_WIDTH$} │ {result}")
        };
        match status {
            ResultStatus::Passed | ResultStatus::Warning => info_with_fields!(&line),
            ResultStatus::Failed => error_fields!(&line),
        }
    }

    pub(crate) fn finish(&self, passed: bool) {
        if passed {
            info_with_fields!("└─ passed ✅");
        } else {
            error_fields!("└─ failed ❌");
        }
    }
}
