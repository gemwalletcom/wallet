// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemRecentActivityServiceProtocol
import struct Gemstone.GemRecentsSections
import struct Gemstone.GemRecentsViewState
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style

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

    var viewState: GemRecentsViewState {
        service.viewState(assets: recentAssets.map { $0.asset.toGem() }, query: searchQuery)
    }

    func sections(_ state: GemRecentsViewState) -> [ListSection<RecentAsset>] {
        let matching = Set(state.matchingAssetIds)
        return DateSectionBuilder(items: recentAssets.filter { matching.contains($0.asset.id.identifier) }, dateKeyPath: \.createdAt).build()
    }

    func emptyModel(_ sections: GemRecentsSections) -> any EmptyContentViewable {
        EmptyContentTypeViewModel(type: EmptyContentType(sections.empty ?? .recents))
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
