// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import GemstoneServices
import Components
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class AssetsResultsSceneViewModel: AssetActions, PerpetualPinActions {
    public static let defaultLimit = 100

    let assetsEnabler: any AssetsEnabler
    private let preferences: Preferences
    private let searchService: WalletSearchService
    let perpetualService: PerpetualService
    private let activityService: ActivityService
    let wallet: Wallet

    let title: String
    let onSelectAssetAction: AssetAction

    public let searchQuery: ObservableQuery<WalletSearchRequest>
    var searchResult: WalletSearchResult {
        searchQuery.value
    }

    var isPresentingToastMessage: ToastMessage?
    private var state: StateViewType<Bool> = .loading

    public init(
        wallet: Wallet,
        assetsEnabler: any AssetsEnabler,
        preferences: Preferences,
        searchService: WalletSearchService,
        perpetualService: PerpetualService,
        activityService: ActivityService,
        request: WalletSearchRequest,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) {
        self.wallet = wallet
        self.assetsEnabler = assetsEnabler
        self.preferences = preferences
        self.searchService = searchService
        self.perpetualService = perpetualService
        self.activityService = activityService
        self.title = title
        searchQuery = ObservableQuery(request, initialValue: .empty)
        onSelectAssetAction = onSelectAsset
    }

    var currencyCode: String {
        preferences.currency
    }

    var sections: WalletSearchSections {
        .from(searchResult)
    }

    var showPinned: Bool {
        sections.pinnedAssets.isNotEmpty
    }

    var showAssets: Bool {
        sections.assets.isNotEmpty
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var perpetuals: [PerpetualData] {
        sections.perpetuals
    }

    var showPerpetuals: Bool {
        searchQuery.request.scope.isList && sections.perpetuals.isNotEmpty && preferences.showPerpetuals(for: wallet)
    }

    var showEmpty: Bool {
        !showPinned && !showAssets && !showPerpetuals
    }

    var searchState: SearchContentState {
        guard showEmpty else { return .results }
        return state.isLoading ? .loading : .empty(.search(type: .assets))
    }

    func contextMenuItems(for assetData: AssetData) -> [ContextMenuItemType] {
        AssetContextMenu.items(
            for: assetData,
            onCopy: { [weak self] in
                self?.isPresentingToastMessage = .copy(
                    CopyTypeViewModel(type: .address(assetData.asset, address: $0), copyValue: $0).message,
                )
            },
            onPin: { [weak self] in
                self?.onPinAsset(assetData.asset, value: !assetData.metadata.isPinned)
            },
            onAddToWallet: { [weak self] in
                self?.onAddToWallet(assetData.asset.id)
            },
        )
    }
}

// MARK: - Actions

extension AssetsResultsSceneViewModel {
    func fetch() {
        Task { await refresh() }
    }

    func refresh() async {
        state = .loading
        do {
            try await searchService.search(
                wallet: wallet,
                query: searchQuery.request.searchBy,
                scope: searchQuery.request.scope,
            )
            state = .data(true)
        } catch {
            state.setError(error)
        }
    }

    func onSelectAsset(_ asset: Asset) {
        onSelectAssetAction?(asset)
        do {
            try activityService.updateRecent(data: .search(asset), walletId: wallet.id)
        } catch {
            debugLog("AssetsResultsSceneViewModel update recent error: \(error)")
        }
    }
}
