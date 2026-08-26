// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import ChainService
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public final class AssetsService: Sendable {
    public let assetStore: AssetStore
    let balanceStore: BalanceStore
    let priceStore: PriceStore
    let assetsProvider: any GemAssetsServiceProtocol
    let chainServiceFactory: any ChainServiceFactorable

    public init(
        assetStore: AssetStore,
        balanceStore: BalanceStore,
        priceStore: PriceStore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsProvider: any GemAssetsServiceProtocol,
    ) {
        self.assetStore = assetStore
        self.balanceStore = balanceStore
        self.priceStore = priceStore
        self.chainServiceFactory = chainServiceFactory
        self.assetsProvider = assetsProvider
    }

    public func addAssets(assets: [AssetBasic]) throws {
        try assetStore.add(assets: assets)
    }

    func getAsset(for assetId: AssetId) throws -> Asset {
        if let asset = try assetStore.getAssets(for: [assetId.identifier]).first {
            return asset
        }
        throw AnyError("asset not found")
    }

    public func getOrFetchAsset(for assetId: AssetId) async throws -> Asset {
        if let asset = try assetStore.getAssets(for: [assetId.identifier]).first {
            return asset
        }
        try await prefetchAssets(assetIds: [assetId])
        return try getAsset(for: assetId)
    }

    public func getOrFetchTokenAsset(for assetId: AssetId) async throws -> Asset {
        if let asset = try assetStore.getAssets(for: [assetId.identifier]).first {
            return asset
        }

        guard let tokenId = assetId.tokenId else {
            return try await getOrFetchAsset(for: assetId)
        }

        let asset = try await chainServiceFactory.service(for: assetId.chain).getTokenData(tokenId: tokenId)
        try addAssets(assets: [asset.defaultBasic])
        return asset
    }

    public func getAssets(for assetIds: [AssetId]) throws -> [Asset] {
        try assetStore.getAssets(for: assetIds.ids)
    }

    public func addBalancesIfMissing(walletId: WalletId, assetIds: [AssetId]) throws {
        for assetId in assetIds {
            try addBalanceIfMissing(walletId: walletId, assetId: assetId)
        }
    }

    public func getBalanceAssetIds(
        walletId: WalletId,
        assetIds: [AssetId],
        filters: [BalanceRequestFilter] = []
    ) throws -> [AssetId] {
        try balanceStore.getBalanceAssetIds(walletId: walletId, assetIds: assetIds, filters: filters)
    }

    @discardableResult
    public func prefetchAssets(assetIds: [AssetId]) async throws -> [AssetId] {
        try await assetsProvider.prefetchAssets(assetIds: assetIds.ids).map { try AssetId(id: $0) }
    }

    public func addBalanceIfMissing(walletId: WalletId, assetId: AssetId) throws {
        try balanceStore.addBalance(assetIds: [assetId], isEnabled: false, for: walletId)
    }

    public func updateEnabled(walletId: WalletId, assetIds: [AssetId], enabled: Bool) throws {
        try balanceStore.setIsEnabled(walletId: walletId, assetIds: assetIds, value: enabled)
    }

    public func updatePinned(walletId: WalletId, assetId: AssetId, pinned: Bool) throws {
        try balanceStore.pinAsset(walletId: walletId, assetId: assetId, value: pinned)
    }

    @discardableResult
    public func updateAsset(assetId: AssetId, currency: String) async throws -> AssetFull {
        let asset = try await getAsset(assetId: assetId)
        try assetStore.add(assets: [asset.basic])
        try assetStore.updateLinks(assetId: assetId, asset.links)
        try assetStore.updateAssociations(assetId: assetId, associations: asset.associations)
        let price = asset.price?.mapToAssetPrice(assetId: assetId) ?? .empty(assetId: assetId)
        try priceStore.updatePrice(price: price, currency: currency)
        if let market = asset.market {
            let rate = try priceStore.getRate(currency: currency).rate
            try priceStore.updateMarket(
                assetId: assetId.identifier,
                market: market,
                rate: rate,
            )
        }
        return asset
    }

    public func addAssets(assetIds: [AssetId]) async throws {
        let assets = try await getAssets(assetIds: assetIds)
        try assetStore.add(assets: assets)
    }

    public func getAsset(assetId: AssetId) async throws -> AssetFull {
        try await AssetFull(assetsProvider.getAsset(assetId: assetId.identifier))
    }

    public func getAssets(assetIds: [AssetId]) async throws -> [AssetBasic] {
        try await assetsProvider
            .getAssets(assetIds: assetIds.ids, currency: nil)
            .map { try AssetBasic($0) }
    }

    // search

    public func searchAssets(query: String, chains: [Chain]) async throws -> [AssetBasic] {
        async let apiAssets = assetsProvider.searchAssets(query: query, chains: chains.map(\.rawValue)).map { try AssetBasic($0) }
        async let networkAssets = searchNetworkAsset(tokenId: query, chains: chains.isEmpty ? Chain.allCases : chains)
        return try await apiAssets + networkAssets
    }

    func searchNetworkAsset(tokenId: String, chains: [Chain]) async -> [AssetBasic] {
        await withTaskGroup(of: AssetBasic?.self) { group in
            for chain in chains {
                group.addTask {
                    let service = self.chainServiceFactory.service(for: chain)
                    guard (try? await service.getIsTokenAddress(tokenId: tokenId)) == true,
                          let asset = try? await service.getTokenData(tokenId: tokenId)
                    else { return nil }

                    return asset.defaultBasic
                }
            }
            return await group.reduce(into: [AssetBasic]()) { if let asset = $1 { $0.append(asset) } }
        }
    }

    public func setSwappableAssets(for chains: [Chain]) throws {
        try assetStore.setAssetIsSwappable(for: chains.map(\.id), value: true)
    }
}
