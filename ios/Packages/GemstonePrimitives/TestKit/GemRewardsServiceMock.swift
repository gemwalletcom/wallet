// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemRewardsServiceMock: GemRewardsServiceProtocol, @unchecked Sendable {
    public var rewardsResult: Result<Rewards, Error> = .success(.mock())
    public var useReferralCodeError: Error?
    public var redeemError: Error?

    public private(set) var rewardsCalls: [WalletId] = []
    public private(set) var usedReferralCodes: [(walletId: String, code: String)] = []
    public private(set) var redeemedIds: [String] = []
    public private(set) var createdReferrals: [String] = []

    public init() {}

    public func createReferral(wallet _: Wallet, code: String) async throws -> Rewards {
        createdReferrals.append(code)
        return try rewardsResult.get()
    }

    public func refresh(walletId: WalletId) async -> GemRewardsResult {
        rewardsCalls.append(walletId)
        guard let rewards = try? rewardsResult.get() else {
            return GemRewardsResult(walletId: walletId, state: .error(error: .Api(msg: "offline")), rewards: nil)
        }
        return GemRewardsResult(walletId: walletId, state: .data, rewards: rewards)
    }

    public func redeem(wallet _: Wallet, redemptionId: String) async throws -> RedemptionResult {
        redeemedIds.append(redemptionId)
        if let redeemError {
            throw redeemError
        }
        return .mock()
    }

    public func selectedWallet(current: Wallet?, wallets _: [Wallet]) -> Wallet? {
        current
    }

    public func useReferralCode(wallet: Wallet, code: String) async throws -> Rewards {
        usedReferralCodes.append((wallet.id, code))
        if let useReferralCodeError {
            throw useReferralCodeError
        }
        return try rewardsResult.get()
    }

    public func wallets(wallets: [Wallet]) -> [Wallet] {
        wallets
    }
}
