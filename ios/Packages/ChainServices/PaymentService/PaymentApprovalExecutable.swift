// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public protocol PaymentApprovalExecutable: Sendable {
    func getApprovalFee(assetId: AssetId, approval: ApprovalData, wallet: Wallet) async throws -> BigInt
    func approve(assetId: AssetId, approval: ApprovalData, wallet: Wallet) async throws -> String
    func waitForApproval(hash: String, assetId: AssetId, wallet: Wallet) async throws
}
