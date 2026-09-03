// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
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
    private let service: any GemRecentActivityServiceProtocol
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
        service: any GemRecentActivityServiceProtocol,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.walletId = walletId
        self.service = service
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
        let matching = Set(recentAssets.map(\.asset).matching(query: searchQuery).map(\.id))
        return recentAssets.filter { matching.contains($0.asset.id) }
    }
}

// MARK: - Actions

extension RecentsSceneViewModel {
    func onSelectClear() {
        Task { [service, types = query.request.types] in
            do {
                try await service.clear(types: types.map { $0.map() })
            } catch {
                debugLog("RecentsSceneViewModel clear error: \(error)")
            }
        }
    }
}
