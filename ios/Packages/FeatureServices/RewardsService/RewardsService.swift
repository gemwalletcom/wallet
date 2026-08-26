// Copyright (c). Gem Wallet. All rights reserved.

import AuthService
import Foundation
import protocol Gemstone.GemRewardsServiceProtocol
import GemstonePrimitives
import Primitives

public protocol RewardsServiceable: Sendable {
    func getRewards(wallet: Wallet) async throws -> Rewards
    func createReferral(wallet: Wallet, code: String) async throws -> Rewards
    func useReferralCode(wallet: Wallet, referralCode: String) async throws
    func generateReferralLink(code: String) -> URL
    func redeem(wallet: Wallet, redemptionId: String) async throws -> RedemptionResult
}

public struct RewardsService: RewardsServiceable, Sendable {
    private let apiService: any GemRewardsServiceProtocol
    private let authService: AuthServiceable

    public init(
        apiService: any GemRewardsServiceProtocol,
        authService: AuthServiceable,
    ) {
        self.apiService = apiService
        self.authService = authService
    }

    public func getRewards(wallet: Wallet) async throws -> Rewards {
        try await Rewards(apiService.getRewards(walletId: wallet.id.id))
    }

    public func useReferralCode(wallet: Wallet, referralCode: String) async throws {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        try await apiService.useReferralCode(walletId: wallet.id.id, auth: auth.json(), code: referralCode)
    }

    public func createReferral(wallet: Wallet, code: String) async throws -> Rewards {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        return try await Rewards(apiService.createReferral(walletId: wallet.id.id, auth: auth.json(), code: code))
    }

    public func generateReferralLink(code: String) -> URL {
        URL(string: "\(Constants.App.website)/join?code=\(code)")!
    }

    public func redeem(wallet: Wallet, redemptionId: String) async throws -> RedemptionResult {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        return try await RedemptionResult(apiService.redeem(walletId: wallet.id.id, auth: auth.json(), redemptionId: redemptionId))
    }
}
