// Copyright (c). Gem Wallet. All rights reserved.

import Style
import protocol Gemstone.GemRecentActivityServiceProtocol
import GemstoneServices
import Components
import struct Gemstone.GemRecentsSections
import struct Gemstone.GemRecentsViewState
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

    func listItem(for asset: Asset) -> ListItemModel {
        let assetModel = AssetViewModel(asset: asset)
        return ListItemModel(
            title: assetModel.name,
            titleStyle: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            imageStyle: .asset(assetImage: assetModel.assetImage),
        )
    }

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
        recentsSections.showsEmpty || recentsSections.showsNoResults
    }

    var showClear: Bool {
        recentsSections.showsClear
    }

    private var viewState: GemRecentsViewState {
        service.viewState(assets: recentAssets.map { $0.asset.toGem() }, query: searchQuery)
    }

    private var recentsSections: GemRecentsSections {
        viewState.sections
    }

    var sections: [ListSection<RecentAsset>] {
        DateSectionBuilder(items: filteredAssets, dateKeyPath: \.createdAt).build()
    }

    var emptyModel: any EmptyContentViewable {
        recentsSections.showsNoResults ? EmptyContentTypeViewModel(type: .search(type: .assets)) : EmptyContentTypeViewModel(type: .recents)
    }

    private var filteredAssets: [RecentAsset] {
        let matching = Set(viewState.matchingAssetIds)
        return recentAssets.filter { matching.contains($0.asset.id.identifier) }
    }
}

// MARK: - Actions

extension RecentsSceneViewModel {
    func onSelectClear() {
        Task { [service, types = query.request.types] in
            do {
                try await service.clear(types: types.map { $0.toGem() })
            } catch {
                debugLog("RecentsSceneViewModel clear error: \(error)")
            }
        }
    }
}
