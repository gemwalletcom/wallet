// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BalanceService
import Foundation
import GemAPI
import NFTService
import Preferences
import Primitives
import TransactionsService

public struct AssetDiscoveryService: AssetDiscoverable {
    private let assetsListService: any GemAPIAssetsListService
    private let assetService: AssetsService
    private let assetsEnabler: any AssetsEnabler
    private let transactionsService: TransactionsService
    private let nftService: NFTService

    public init(
        assetsListService: any GemAPIAssetsListService,
        assetService: AssetsService,
        assetsEnabler: any AssetsEnabler,
        transactionsService: TransactionsService,
        nftService: NFTService,
    ) {
        self.assetsListService = assetsListService
        self.assetService = assetService
        self.assetsEnabler = assetsEnabler
        self.transactionsService = transactionsService
        self.nftService = nftService
    }

    public func discoverAssets(wallet: Wallet) async throws {
        let preferences = WalletPreferences(walletId: wallet.walletId)

        try await discoverAssets(wallet: wallet, preferences: preferences)
        try await discoverTransactions(wallet: wallet, preferences: preferences)
        try await discoverNFTs(wallet: wallet, preferences: preferences)
    }

    private func discoverAssets(wallet: Wallet, preferences: WalletPreferences) async throws {
        let assetIds = try await assetsListService.getDeviceAssets(walletId: wallet.id, fromTimestamp: preferences.assetsTimestamp)
        if assetIds.isNotEmpty {
            try await assetService.prefetchAssets(assetIds: assetIds)
            try await assetsEnabler.enableAssets(wallet: wallet, assetIds: assetIds, enabled: true)
        }

        preferences.completeInitialLoadAssets = true
        preferences.assetsTimestamp = Int(Date.now.timeIntervalSince1970)
    }

    private func discoverTransactions(wallet: Wallet, preferences: WalletPreferences) async throws {
        guard !preferences.completeInitialLoadTransactions else { return }
        try await transactionsService.updateAll(walletId: wallet.walletId)
        preferences.completeInitialLoadTransactions = true
    }

    private func discoverNFTs(wallet: Wallet, preferences: WalletPreferences) async throws {
        guard !preferences.completeInitialLoadNFTs else { return }
        try await nftService.updateAssets(wallet: wallet)
        preferences.completeInitialLoadNFTs = true
    }
}
