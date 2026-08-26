// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class RecentsSceneViewModel {
    private let recentActivityStore: RecentActivityStore
    private let walletId: WalletId

    public let query: ObservableQuery<RecentActivityRequest>
    public let onSelect: (Asset) -> Void

    var searchQuery: String = ""

    public var recentAssets: [RecentAsset] {
        query.value
    }

    public init(
        walletId: WalletId,
        types: [RecentActivityType],
        filters: [AssetsRequestFilter] = [],
        recentActivityStore: RecentActivityStore,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.walletId = walletId
        self.recentActivityStore = recentActivityStore
        query = ObservableQuery(RecentActivityRequest(walletId: walletId, limit: .max, types: types, filters: filters), initialValue: [])
        self.onSelect = onSelect
    }

    var title: String {
        Localized.RecentActivity.title
    }

    var clearTitle: String {
        Localized.Filter.clear
    }

    var showEmpty: Bool {
        recentAssets.isEmpty || (!searchQuery.isEmpty && filteredAssets.isEmpty)
    }

    var showClear: Bool {
        recentAssets.isNotEmpty
    }

    var sections: [ListSection<RecentAsset>] {
        DateSectionBuilder(items: filteredAssets, dateKeyPath: \.createdAt).build()
    }

    var emptyModel: any EmptyContentViewable {
        if recentAssets.isEmpty {
            return EmptyContentTypeViewModel(type: .recents)
        }
        return EmptyContentTypeViewModel(type: .search(type: .assets))
    }

    private var filteredAssets: [RecentAsset] {
        guard !searchQuery.isEmpty else { return recentAssets }
        let chains = Set(recentAssets.map(\.asset.chain).filter(query: searchQuery))
        return recentAssets.filter {
            $0.asset.name.localizedCaseInsensitiveContains(searchQuery) ||
                $0.asset.symbol.localizedCaseInsensitiveContains(searchQuery) ||
                chains.contains($0.asset.chain)
        }
    }
}

// MARK: - Actions

extension RecentsSceneViewModel {
    func onSelectClear() {
        do {
            try recentActivityStore.clear(walletId: walletId, types: query.request.types)
        } catch {
            debugLog("RecentsSceneViewModel clear error: \(error)")
        }
    }
}
