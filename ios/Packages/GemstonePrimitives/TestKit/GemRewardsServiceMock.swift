// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemRewardsServiceMock: GemRewardsServiceProtocol, @unchecked Sendable {
    public var rewardsResult: Result<Rewards, Error> = .success(.mock())
    public var stateForRewards: (Rewards?) -> GemRewardsState = { rewards in .mock(referralCode: rewards?.code, referralLink: rewards?.code.map { "https://gemwallet.com/join?code=\($0)" }) }
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

    public func getRewards(walletId: WalletId) async throws -> Rewards {
        rewardsCalls.append(walletId)
        return try rewardsResult.get()
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

    public func state(rewards: Rewards?) -> GemRewardsState {
        stateForRewards(rewards)
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
