// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitivesTestKit

final class GemRewardsServiceMock: GemRewardsServiceProtocol, @unchecked Sendable {
    var selectedWalletValue: Wallet?
    var walletsValue: [Wallet] = []
    var rewardsResult: Result<Rewards, Error> = .success(.mock())
    var stateForRewards: (Rewards?) -> GemRewardsState = { rewards in .mock(referralCode: rewards?.code, referralLink: rewards?.code.map { "https://gemwallet.com/join?code=\($0)" }) }
    var useReferralCodeError: Error?
    var redeemError: Error?

    private(set) var rewardsCalls: [WalletId] = []
    private(set) var usedReferralCodes: [(walletId: String, code: String)] = []
    private(set) var redeemedIds: [String] = []
    private(set) var createdReferrals: [String] = []

    func createReferral(wallet: Wallet, code: String) async throws -> Rewards {
        createdReferrals.append(code)
        _ = wallet
        return try rewardsResult.get()
    }

    func getRewards(walletId: WalletId) async throws -> Rewards {
        rewardsCalls.append(walletId)
        return try rewardsResult.get()
    }

    func redeem(wallet _: Wallet, redemptionId: String) async throws -> RedemptionResult {
        redeemedIds.append(redemptionId)
        if let redeemError { throw redeemError }
        return .mock()
    }

    func selectedWallet(current: Wallet?, wallets _: [Wallet]) -> Wallet? {
        selectedWalletValue ?? current
    }

    func state(rewards: Rewards?) -> GemRewardsState {
        stateForRewards(rewards)
    }

    func useReferralCode(wallet: Wallet, code: String) async throws {
        usedReferralCodes.append((wallet.id, code))
        if let useReferralCodeError { throw useReferralCodeError }
    }

    func wallets(wallets: [Wallet]) -> [Wallet] {
        walletsValue.isEmpty ? wallets : walletsValue
    }
}
