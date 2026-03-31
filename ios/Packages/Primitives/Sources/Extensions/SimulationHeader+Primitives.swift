public extension SimulationHeader {
    var approvalValue: ApprovalValue? {
        ApprovalValue(value: value, isUnlimited: isUnlimited)
    }
}
