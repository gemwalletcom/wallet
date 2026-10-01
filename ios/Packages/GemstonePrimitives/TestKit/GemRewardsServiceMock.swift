// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public final class GemRewardsServiceMock: GemRewardsServiceProtocol, @unchecked Sendable {
    public var rewardsResult: Result<Rewards, Error> = .success(.mock(
        code: "test123",
        inviteRewardPoints: 100,
        referralCount: 5,
        status: .verified,
        referralAllowance: .mock(daily: .mock(limit: 5, available: 5), weekly: .mock(limit: 20, available: 20)),
    ))
    public var useReferralCodeError: Error?
    public var redeemError: Error?

    public private(set) var rewardsCalls: [Primitives.WalletId] = []
    public private(set) var usedReferralCodes: [(walletId: Primitives.WalletId, code: String)] = []
    public private(set) var redeemedIds: [String] = []
    public private(set) var createdReferrals: [String] = []

    public init() {}

    public func createReferral(walletId _: Primitives.WalletId, code: String) async throws -> Rewards {
        createdReferrals.append(code)
        return try rewardsResult.get()
    }

    public func refresh(walletId: Primitives.WalletId) async -> GemRewardsResult {
        rewardsCalls.append(walletId)
        guard let rewards = try? rewardsResult.get() else {
            return GemRewardsResult(walletId: walletId, state: .error(error: .Api(msg: "offline")), rewards: nil)
        }
        return GemRewardsResult(walletId: walletId, state: .data, rewards: rewards)
    }

    public func redeem(walletId _: Primitives.WalletId, redemptionId: String) async throws -> RedemptionResult {
        redeemedIds.append(redemptionId)
        if let redeemError {
            throw redeemError
        }
        return .mock()
    }

    public func useReferralCode(walletId: Primitives.WalletId, code: String) async throws -> Rewards {
        usedReferralCodes.append((walletId, code))
        if let useReferralCodeError {
            throw useReferralCodeError
        }
        return try rewardsResult.get()
    }
}
