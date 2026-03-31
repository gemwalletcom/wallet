public extension SimulationWarningType {
    var approvalValue: ApprovalValue? {
        guard let rawValue = approvalRawValue else { return nil }
        return ApprovalValue(value: rawValue ?? "0", isUnlimited: rawValue == nil)
    }

    private var approvalRawValue: Optional<String?> {
        switch self {
        case .tokenApproval(let approval): .some(approval.value)
        case .permitApproval(let approval): .some(approval.value)
        case .permitBatchApproval(let value): .some(value)
        default: .none
        }
    }
}
