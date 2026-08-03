// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import PaymentService
import Primitives

public final class PaymentApprovalExecutableMock: PaymentApprovalExecutable, @unchecked Sendable {
    public private(set) var approvals: [ApprovalData] = []
    public private(set) var confirmedHashes: [String] = []
    private let hash: String
    private let fee: BigInt
    private let validationError: (any Error)?

    public init(hash: String = "0xapproval", fee: BigInt = .zero, validationError: (any Error)? = .none) {
        self.hash = hash
        self.fee = fee
        self.validationError = validationError
    }

    public func getApprovalFee(assetId _: AssetId, approval _: ApprovalData, wallet _: Wallet) async throws -> BigInt {
        if let validationError {
            throw validationError
        }
        return fee
    }

    public func approve(assetId _: AssetId, approval: ApprovalData, wallet _: Wallet) async throws -> String {
        approvals.append(approval)
        return hash
    }

    public func waitForApproval(hash: String, assetId _: AssetId, wallet _: Wallet) async throws {
        confirmedHashes.append(hash)
    }
}
