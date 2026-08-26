// Copyright (c). Gem Wallet. All rights reserved.

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
    private let service: any GemRewardsServiceProtocol
    private let authService: AuthServiceable

    public init(
        service: any GemRewardsServiceProtocol,
        authService: AuthServiceable,
    ) {
        self.service = service
        self.authService = authService
    }

    public func getRewards(wallet: Wallet) async throws -> Rewards {
        try await Rewards(service.getRewards(walletId: wallet.id.id))
    }

    public func useReferralCode(wallet: Wallet, referralCode: String) async throws {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        try await service.useReferralCode(walletId: wallet.id.id, auth: auth.json(), code: referralCode)
    }

    public func createReferral(wallet: Wallet, code: String) async throws -> Rewards {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        return try await Rewards(service.createReferral(walletId: wallet.id.id, auth: auth.json(), code: code))
    }

    public func generateReferralLink(code: String) -> URL {
        URL(string: service.referralLink(code: code))!
    }

    public func redeem(wallet: Wallet, redemptionId: String) async throws -> RedemptionResult {
        let auth = try await authService.getAuthPayload(wallet: wallet)
        return try await RedemptionResult(service.redeem(walletId: wallet.id.id, auth: auth.json(), redemptionId: redemptionId))
    }
}
