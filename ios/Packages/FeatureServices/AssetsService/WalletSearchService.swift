// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Preferences
import Primitives
import Store

public struct WalletSearchService: Sendable {
    private let assetsService: AssetsService
    private let searchStore: SearchStore
    private let perpetualStore: PerpetualStore
    private let assetListStore: AssetListStore
    private let priceStore: PriceStore
    private let preferences: Preferences
    private let searchProvider: any GemAPISearchService

    public init(
        assetsService: AssetsService,
        searchStore: SearchStore,
        perpetualStore: PerpetualStore,
        assetListStore: AssetListStore,
        priceStore: PriceStore,
        preferences: Preferences,
        searchProvider: any GemAPISearchService = GemAPIService.shared,
    ) {
        self.assetsService = assetsService
        self.searchStore = searchStore
        self.perpetualStore = perpetualStore
        self.assetListStore = assetListStore
        self.priceStore = priceStore
        self.preferences = preferences
        self.searchProvider = searchProvider
    }

    public func search(wallet: Wallet, query: String, scope: WalletSearchTag = .all) async throws {
        let scopeChains = WalletSearchScope.chains(for: wallet)
        let chains = scope.isAll ? (scopeChains.isEmpty ? Chain.allCases : scopeChains) : []

        async let networkAssets = assetsService.searchNetworkAsset(tokenId: query, chains: chains)
        async let searchResult = searchProvider.search(query: query, chains: scopeChains, tags: [scope.apiTag].compactMap(\.self))
        let assets = try await searchResult.assets + networkAssets

        let searchKey = scope.searchKey(query: query)
        try store(assets: assets, wallet: wallet, searchKey: searchKey)
        if scope.includesPerpetuals {
            try await store(perpetuals: searchResult.perpetuals, searchKey: searchKey)
        }
        if scope.isAll {
            try await store(lists: searchResult.lists, searchKey: searchKey)
        }
    }
}

// MARK: - Private

private extension WalletSearchService {
    func store(assets: [AssetBasic], wallet: Wallet, searchKey: String) throws {
        try assetsService.addAssets(assets: assets)
        try priceStore.updatePrices(prices: prices(from: assets), currency: preferences.currency)
        try assetsService.addBalancesIfMissing(walletId: wallet.id, assetIds: assets.map(\.asset.id))
        try searchStore.add(type: .asset, query: searchKey, ids: assets.map(\.asset.id.identifier))
    }

    func store(perpetuals: [PerpetualSearchData], searchKey: String) throws {
        try assetsService.addAssets(assets: perpetuals.map(\.assetBasic))
        try perpetualStore.upsertPerpetuals(perpetuals.map(\.perpetual))
        try searchStore.add(type: .perpetual, query: searchKey, ids: perpetuals.map(\.perpetual.id.identifier))
    }

    func store(lists: [AssetList], searchKey: String) throws {
        try assetListStore.upsert(lists)
        try searchStore.add(type: .list, query: searchKey, ids: lists.map(\.id))
    }

    func prices(from assets: [AssetBasic]) -> [AssetPrice] {
        assets.compactMap { asset in
            guard let price = asset.price else { return nil }
            return AssetPrice(
                assetId: asset.asset.id,
                price: price.price,
                priceChangePercentage24h: price.priceChangePercentage24h,
                updatedAt: price.updatedAt,
            )
        }
    }
}
