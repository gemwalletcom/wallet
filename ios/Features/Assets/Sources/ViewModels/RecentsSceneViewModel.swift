// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.assetText
import protocol Gemstone.GemRecentActivityServiceProtocol
import struct Gemstone.GemRecentsDay
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

    public let query: ObservableQuery<RecentActivityQuery>
    public let onSelect: (Asset) -> Void

    var searchQuery: String = ""

    func listItem(for asset: Asset) -> ListItemModel {
        ListItemModel(
            title: asset.name,
            titleStyle: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            imageStyle: .asset(assetImage: AssetImage(icon: assetText(asset: asset.toGem()).icon)),
        )
    }

    public var recentAssets: [RecentAsset] {
        query.value
    }

    public init(
        walletId: WalletId,
        types: [RecentActivityType],
        filters: [AssetsQueryFilter] = [],
        service: any GemRecentActivityServiceProtocol,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.service = service
        query = ObservableQuery(RecentActivityQuery(walletId: walletId, limit: .max, types: types, filters: filters), initialValue: [])
        self.onSelect = onSelect
    }

    var title: String {
        Localized.RecentActivity.title
    }

    var clearTitle: String {
        Localized.Filter.clear
    }

    var viewState: GemRecentsViewState {
        service.viewState(recents: recentAssets.map { $0.toGem() }, days: recentAssets.map(\.createdAt.gemDay), query: searchQuery)
    }

    func title(for day: GemRecentsDay) -> String {
        TransactionDateFormatter(date: day.day.date).section
    }

    func emptyModel(_ sections: GemRecentsSections) -> any EmptyContentViewable {
        EmptyStateViewModel(kind: sections.empty ?? .recents)
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
