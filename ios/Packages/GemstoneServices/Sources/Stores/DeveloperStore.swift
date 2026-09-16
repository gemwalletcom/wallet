// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.BannerState
import protocol Gemstone.GemDeveloperStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneDeveloperStore: GemDeveloperStore, @unchecked Sendable {
    private let transactionStore: TransactionStore
    private let assetStore: AssetStore
    private let stakeStore: StakeStore
    private let bannerStore: BannerStore
    private let priceStore: PriceStore

    public init(
        transactionStore: TransactionStore,
        assetStore: AssetStore,
        stakeStore: StakeStore,
        bannerStore: BannerStore,
        priceStore: PriceStore,
    ) {
        self.transactionStore = transactionStore
        self.assetStore = assetStore
        self.stakeStore = stakeStore
        self.bannerStore = bannerStore
        self.priceStore = priceStore
    }

    public func clearTransactions() async throws {
        _ = try transactionStore.clear()
    }

    public func clearTokens() async throws {
        _ = try assetStore.clearTokens()
    }

    public func clearDelegations() async throws {
        _ = try stakeStore.clearDelegations()
    }

    public func clearValidators() async throws {
        _ = try stakeStore.clearValidators()
    }

    public func clearPrices() async throws {
        _ = try priceStore.clear()
    }

    public func clearBanners() async throws {
        _ = try bannerStore.clear()
    }

    public func updateBannerStates(from: Gemstone.BannerState, to: Gemstone.BannerState) async throws {
        _ = try bannerStore.updateStates(from: from.toPrimitives(), to: to.toPrimitives())
    }
}
