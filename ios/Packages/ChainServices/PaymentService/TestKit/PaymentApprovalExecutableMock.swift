// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public final class PaymentApprovalExecutableMock: PaymentApprovalExecutable, @unchecked Sendable {
    public private(set) var confirmedHashes: [String] = []

    public init() {}

    public func waitForApproval(hash: String, assetId _: AssetId, wallet _: Wallet) async throws {
        confirmedHashes.append(hash)
    }
}
