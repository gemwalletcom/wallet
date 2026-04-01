// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import RewardsService

public struct RewardsServiceMock: RewardsServiceable, Sendable {
    public var rewardsResult: Result<Rewards, Error>
    public var createReferralResult: Result<Rewards, Error>
    public var useCodeError: Error?
    public var redemptionOptionResult: Result<RewardRedemptionOption, Error>
    public var redeemResult: Result<RedemptionResult, Error>

    public init(
        rewardsResult: Result<Rewards, Error> = .success(.mock()),
        createReferralResult: Result<Rewards, Error> = .success(.mock()),
        useCodeError: Error? = nil,
        redemptionOptionResult: Result<RewardRedemptionOption, Error> = .success(.mock()),
        redeemResult: Result<RedemptionResult, Error> = .success(.mock()),
    ) {
        self.rewardsResult = rewardsResult
        self.createReferralResult = createReferralResult
        self.useCodeError = useCodeError
        self.redemptionOptionResult = redemptionOptionResult
        self.redeemResult = redeemResult
    }

    public func getRewards(wallet _: Wallet) async throws -> Rewards {
        try rewardsResult.get()
    }

    public func createReferral(wallet _: Wallet, code _: String) async throws -> Rewards {
        try createReferralResult.get()
    }

    public func useReferralCode(wallet _: Wallet, referralCode _: String) async throws {
        if let error = useCodeError {
            throw error
        }
    }

    public func generateReferralLink(code: String) -> URL {
        URL(string: "\(Constants.App.website)/join?code=\(code)")!
    }

    public func getRedemptionOption(code _: String) async throws -> RewardRedemptionOption {
        try redemptionOptionResult.get()
    }

    public func redeem(wallet _: Wallet, redemptionId _: String) async throws -> RedemptionResult {
        try redeemResult.get()
    }
}

public extension RewardsService {
    static func mock() -> RewardsServiceMock {
        RewardsServiceMock()
    }
}
