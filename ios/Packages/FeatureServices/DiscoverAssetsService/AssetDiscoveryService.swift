// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GemstonePrimitives
import NFTService
import Preferences
import Primitives
import TransactionsService

public struct AssetDiscoveryService: AssetDiscoverable {
    private let discovery: any GemAssetDiscoveryServiceProtocol
    private let transactionsService: TransactionsService
    private let nftService: NFTService
    private let preferences: Preferences

    public init(
        discovery: any GemAssetDiscoveryServiceProtocol,
        transactionsService: TransactionsService,
        nftService: NFTService,
        preferences: Preferences,
    ) {
        self.discovery = discovery
        self.transactionsService = transactionsService
        self.nftService = nftService
        self.preferences = preferences
    }

    public func discoverAssets(wallet: Wallet) async throws {
        let preferences = WalletPreferences(walletId: wallet.id)

        async let assets: () = discoverAssets(wallet: wallet, preferences: preferences)
        async let transactions: () = discoverTransactions(wallet: wallet, preferences: preferences)
        async let nfts: () = discoverNFTs(wallet: wallet, preferences: preferences)
        _ = try await (assets, transactions, nfts)
    }

    private func discoverAssets(wallet: Wallet, preferences: WalletPreferences) async throws {
        _ = try await discovery.discover(walletId: wallet.id.id, currency: Currency(id: self.preferences.currency).json())
        preferences.completeInitialLoadAssets = true
    }

    private func discoverTransactions(wallet: Wallet, preferences: WalletPreferences) async throws {
        guard !preferences.completeInitialLoadTransactions else { return }
        try await transactionsService.updateAll(walletId: wallet.id)
        preferences.completeInitialLoadTransactions = true
    }

    private func discoverNFTs(wallet: Wallet, preferences: WalletPreferences) async throws {
        guard !preferences.completeInitialLoadNFTs else { return }
        try await nftService.updateAssets(wallet: wallet)
        preferences.completeInitialLoadNFTs = true
    }
}
