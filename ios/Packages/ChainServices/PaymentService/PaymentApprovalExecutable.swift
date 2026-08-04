// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PaymentApprovalExecutable: Sendable {
    func waitForApproval(hash: String, assetId: AssetId, wallet: Wallet) async throws
}
