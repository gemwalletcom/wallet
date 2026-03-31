
public extension ApprovalData {
    var approvalValue: ApprovalValue? {
        ApprovalValue(value: value, isUnlimited: isUnlimited)
    }
}
