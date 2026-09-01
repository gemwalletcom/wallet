// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import GemstoneServices
import Foundation
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class RecentAssetsModel {
    private static let sectionLimit: Int = 10

    private let walletId: WalletId
    private let recentAssetsService: any GemRecentActivityServiceProtocol

    public let query: ObservableQuery<RecentActivityRequest>
    public var isPresenting: Bool = false

    public init(
        walletId: WalletId,
        types: [RecentActivityType],
        filters: [AssetsRequestFilter] = [],
        recentAssetsService: any GemRecentActivityServiceProtocol,
    ) {
        self.walletId = walletId
        self.recentAssetsService = recentAssetsService
        query = ObservableQuery(
            RecentActivityRequest(
                walletId: walletId,
                limit: Self.sectionLimit,
                types: types,
                filters: filters,
            ),
            initialValue: [],
        )
    }

    public var assets: [RecentAsset] { query.value }
    public var assetModels: [AssetViewModel] { assets.map { AssetViewModel(asset: $0.asset) }}
    public var hasAssets: Bool { assets.isNotEmpty }

    public func recentModel(onSelect: @escaping (Asset) -> Void) -> RecentsSceneViewModel {
        RecentsSceneViewModel(
            walletId: walletId,
            types: query.request.types,
            filters: query.request.filters,
            recentAssetsService: recentAssetsService,
            onSelect: onSelect,
        )
    }
}

// MARK: - Actions

public extension RecentAssetsModel {
    func present() {
        isPresenting = true
    }

    func dismiss() {
        isPresenting = false
    }
}
