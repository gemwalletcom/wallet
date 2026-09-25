// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAssetAction
import protocol Gemstone.GemRecentActivityServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class RecentAssetsModel {
    private let walletId: WalletId
    private let service: any GemRecentActivityServiceProtocol

    public let query: ObservableQuery<RecentActivityQuery>
    public var isPresenting: Bool = false

    public init(
        walletId: WalletId,
        types: [RecentActivityType],
        filters: [AssetsQueryFilter] = [],
        service: any GemRecentActivityServiceProtocol,
    ) {
        self.walletId = walletId
        self.service = service
        query = ObservableQuery(
            RecentActivityQuery(
                walletId: walletId,
                limit: GemConstants.recentAssetsLimit,
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
            service: service,
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

    func add(action: GemAssetAction, asset: Asset) {
        Task { [service] in
            do {
                try await service.addRecent(action: action, asset: asset.toGem())
            } catch {
                debugLog("Failed to update recent activity: \(error)")
            }
        }
    }
}
