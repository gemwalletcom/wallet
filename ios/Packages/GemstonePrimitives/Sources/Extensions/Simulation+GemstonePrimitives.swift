// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.SimulationHeader
import enum Gemstone.SimulationWarningType
import Primitives

public extension Gemstone.SimulationHeader {
    var approvalValue: ApprovalValue? {
        isUnlimited ? .unlimited : value.map { .exact(BigInt($0)) }
    }
}

public extension Gemstone.SimulationWarningType {
    var approvalValue: ApprovalValue? {
        guard let amount = approvalAmount else { return nil }
        return amount.map(ApprovalValue.exact) ?? .unlimited
    }

    private var approvalAmount: BigInt?? {
        switch self {
        case let .tokenApproval(approval): .some(approval.value)
        case let .permitApproval(approval): .some(approval.value)
        case let .permitBatchApproval(value): .some(value)
        default: .none
        }
    }
}
