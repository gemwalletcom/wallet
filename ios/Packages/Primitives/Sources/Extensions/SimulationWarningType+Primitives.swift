public extension SimulationWarningType {
    var approvalValue: ApprovalValue? {
        guard let rawValue = approvalRawValue else { return nil }
        return ApprovalValue(value: rawValue ?? "0", isUnlimited: rawValue == nil)
    }

    private var approvalRawValue: String?? {
        switch self {
        case let .tokenApproval(approval): .some(approval.value)
        case let .permitApproval(approval): .some(approval.value)
        case let .permitBatchApproval(value): .some(value)
        default: .none
        }
    }
}
