// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemRewardsServiceProtocol
import struct Gemstone.RedemptionResult
import struct Gemstone.Rewards
import Primitives

public extension GemRewardsServiceProtocol {
    func createReferral(wallet: Primitives.Wallet, code: String) async throws -> Rewards {
        try await createReferral(wallet: wallet.toGem(), code: code)
    }

    func useReferralCode(wallet: Primitives.Wallet, code: String) async throws -> Rewards {
        try await useReferralCode(wallet: wallet.toGem(), code: code)
    }

    func redeem(wallet: Primitives.Wallet, redemptionId: String) async throws -> RedemptionResult {
        try await redeem(wallet: wallet.toGem(), redemptionId: redemptionId)
    }
}
