// Copyright (c). Gem Wallet. All rights reserved.

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
    private let recentActivityStore: RecentActivityStore

    public let query: ObservableQuery<RecentActivityRequest>
    public var isPresenting: Bool = false

    public init(
        walletId: WalletId,
        types: [RecentActivityType],
        filters: [AssetsRequestFilter] = [],
        recentActivityStore: RecentActivityStore,
    ) {
        self.walletId = walletId
        self.recentActivityStore = recentActivityStore
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
            recentActivityStore: recentActivityStore,
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
