// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemRewardsServiceProtocol
import Primitives

public extension GemRewardsServiceProtocol {
    func getRewards(wallet: Primitives.Wallet) async throws -> Primitives.Rewards {
        try await Primitives.Rewards(getRewards(walletId: wallet.id.id))
    }

    func createReferral(wallet: Primitives.Wallet, code: String) async throws -> Primitives.Rewards {
        try await Primitives.Rewards(createReferral(wallet: wallet.json(), code: code))
    }

    func useReferralCode(wallet: Primitives.Wallet, code: String) async throws {
        try await useReferralCode(wallet: wallet.json(), code: code)
    }

    func redeem(wallet: Primitives.Wallet, redemptionId: String) async throws -> Primitives.RedemptionResult {
        try await Primitives.RedemptionResult(redeem(wallet: wallet.json(), redemptionId: redemptionId))
    }

    func referralLink(code: String) throws -> URL {
        guard let url = URL(string: referralLink(code: code)) else {
            throw AnyError("invalid referral link")
        }
        return url
    }
}
