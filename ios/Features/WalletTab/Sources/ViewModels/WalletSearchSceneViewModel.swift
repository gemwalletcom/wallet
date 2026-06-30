// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AssetsService
import BalanceService
import Components
import Foundation
import Localization
import PerpetualService
import Preferences
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class WalletSearchSceneViewModel: Sendable {
    private let searchService: WalletSearchService
    private let activityService: ActivityService
    private let assetsEnabler: any AssetsEnabler
    private let balanceService: BalanceService
    private let perpetualService: PerpetualService
    private let preferences: ObservablePreferences

    private let wallet: Wallet
    private let onDismissSearch: VoidAction
    private let onAddToken: VoidAction

    private var state: StateViewType<Bool> = .noData

    var searchModel: WalletSearchModel

    public let searchQuery: ObservableQuery<WalletSearchRequest>
    public let recentsQuery: ObservableQuery<RecentActivityRequest>

    var searchResult: WalletSearchResult {
        searchQuery.value
    }

    var recents: [RecentAsset] {
        recentsQuery.value
    }

    var isPresentingToastMessage: ToastMessage?
    var isSearching: Bool = false
    var isSearchPresented: Bool = false
    var dismissSearch: Bool = false
    var isPresentingRecents: Bool = false

    let onSelectAssetAction: AssetAction

    public init(
        wallet: Wallet,
        searchService: WalletSearchService,
        activityService: ActivityService,
        assetsEnabler: any AssetsEnabler,
        balanceService: BalanceService,
        perpetualService: PerpetualService,
        preferences: ObservablePreferences = .default,
        onDismissSearch: VoidAction,
        onSelectAssetAction: AssetAction,
        onAddToken: VoidAction,
    ) {
        self.wallet = wallet
        self.searchService = searchService
        self.activityService = activityService
        self.assetsEnabler = assetsEnabler
        self.balanceService = balanceService
        self.perpetualService = perpetualService
        self.preferences = preferences
        self.onDismissSearch = onDismissSearch
        self.onSelectAssetAction = onSelectAssetAction
        self.onAddToken = onAddToken
        searchModel = WalletSearchModel(selectType: .manage)

        searchQuery = ObservableQuery(
            WalletSearchRequest(
                walletId: wallet.id,
                limit: WalletSearchModel.initialFetchLimit,
                types: WalletSearchModel.searchItemTypes,
            ),
            initialValue: .empty,
        )
        recentsQuery = ObservableQuery(
            RecentActivityRequest(
                walletId: wallet.id,
                limit: 10,
                types: WalletSearchModel.recentActivityTypes,
            ),
            initialValue: [],
        )
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var assetsTitle: String {
        Localized.Assets.title
    }

    var listsTitle: String {
        Localized.Common.lists
    }

    var sections: WalletSearchSections {
        .from(searchResult)
    }

    var recentModels: [AssetViewModel] {
        recents.map { AssetViewModel(asset: $0.asset) }
    }

    var currencyCode: String {
        preferences.preferences.currency
    }

    var showTags: Bool {
        searchModel.searchableQuery.isEmpty
    }

    var showRecents: Bool {
        searchModel.searchableQuery.isEmpty && recents.isNotEmpty
    }

    var showPerpetuals: Bool {
        sections.perpetuals.isNotEmpty && preferences.showPerpetuals(for: wallet)
    }

    var showLoading: Bool {
        state.isLoading && showEmpty
    }

    var showEmpty: Bool {
        !showRecents && !showPinned && !showAssets && !showPerpetuals && !showLists
    }

    var showPinned: Bool {
        sections.pinnedAssets.isNotEmpty || showPinnedPerpetuals
    }

    var showPinnedPerpetuals: Bool {
        sections.pinnedPerpetuals.isNotEmpty && preferences.showPerpetuals(for: wallet)
    }

    var showAssets: Bool {
        sections.assets.isNotEmpty
    }

    var showLists: Bool {
        sections.lists.isNotEmpty
    }

    var showAddToken: Bool {
        wallet.hasTokenSupport
    }

    var previewAssets: [AssetData] {
        Array(sections.assets.prefix(assetsPreviewLimit))
    }

    var previewPerpetuals: [PerpetualData] {
        Array(sections.perpetuals.prefix(searchModel.perpetualsLimit))
    }

    var hasMoreAssets: Bool {
        searchResult.assets.count > assetsPreviewLimit
    }

    var hasMorePerpetuals: Bool {
        searchResult.perpetuals.count > searchModel.perpetualsLimit
    }

    var recentsModel: RecentsSceneViewModel {
        RecentsSceneViewModel(
            walletId: wallet.id,
            types: recentsQuery.request.types,
            filters: recentsQuery.request.filters,
            activityService: activityService,
            onSelect: onSelectRecent,
        )
    }

    var assetsResultsDestination: Scenes.AssetsResults {
        Scenes.AssetsResults(
            searchQuery: searchQuery.request.searchBy,
            scope: searchQuery.request.scope,
        )
    }

    func listDestination(for list: AssetList) -> Scenes.AssetsResults {
        Scenes.AssetsResults(
            searchQuery: .empty,
            scope: .list(list.id),
            title: list.name,
        )
    }

    func contextMenuItems(for assetData: AssetData) -> [ContextMenuItemType] {
        AssetContextMenu.items(
            for: assetData,
            onCopy: { [weak self] in
                self?.onSelectCopyAddress(CopyTypeViewModel(type: .address(assetData.asset, address: $0), copyValue: $0).message)
            },
            onPin: { [weak self] in
                self?.onSelectPinAsset(assetData, value: !assetData.metadata.isPinned)
            },
            onAddToWallet: { [weak self] in
                self?.onSelectAddToWallet(assetData.asset)
            },
        )
    }
}

// MARK: - Actions

extension WalletSearchSceneViewModel {
    func onAppear() {
        dismissSearch = false
        isSearchPresented = true
    }

    func onSearch(query: String) async {
        let query = query.trim()
        guard !query.isEmpty else { return }

        await search(query: query)
    }

    func onSelectTag(tag: AssetTagSelection) {
        searchModel.tagsViewModel.selectedTag = tag
        searchModel.focus = .tags
        let scope: WalletSearchTag = if let assetTag = tag.tag { .filter(assetTag) } else { .all }
        searchQuery.request.scope = scope
        updateRequest()
        Task {
            await search(query: .empty, scope: scope)
        }
    }

    func onSelectAsset(_ asset: Asset) {
        onSelectAssetAction?(asset)
        updateRecent(asset)
    }

    func onSelectRecents() {
        isPresentingRecents = true
    }

    func onSelectRecent(asset: Asset) {
        onSelectAssetAction?(asset)
        isPresentingRecents = false
    }

    func onSelectAddCustomToken() {
        onAddToken?()
    }

    func onSelectAddToWallet(_ asset: Asset) {
        enableAsset(asset.id)
        isPresentingToastMessage = .addedToWallet()
    }

    func onSelectPinAsset(_ assetData: AssetData, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.id, assetId: assetData.asset.id)
            isPresentingToastMessage = .pin(assetData.asset.name, pinned: value)
            if value, !assetData.metadata.isBalanceEnabled {
                enableAsset(assetData.asset.id)
            }
        } catch {
            debugLog("WalletSearchSceneViewModel pin asset error: \(error)")
        }
    }

    func onSelectPinPerpetual(_ perpetualData: PerpetualData) {
        let pinned = !perpetualData.metadata.isPinned
        do {
            try perpetualService.setPinned(pinned, perpetualId: perpetualData.perpetual.id)
            isPresentingToastMessage = .pin(perpetualData.perpetual.name, pinned: pinned)
        } catch {
            debugLog("WalletSearchSceneViewModel pin perpetual error: \(error)")
        }
    }

    func onSelectCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }

    func onChangeSearchQuery(_: String, _: String) {
        updateRequest()
    }

    func onChangeFocus(_: Bool, isSearching: Bool) {
        if isSearching {
            searchModel.focus = .search
            searchModel.tagsViewModel.selectedTag = AssetTagSelection.all
            searchQuery.request.scope = .all
            updateRequest()
        }
    }

    func onChangeSearchPresented(_: Bool, isPresented: Bool) {
        guard !isPresented else { return }
        dismissSearch = true
        onDismissSearch?()
    }
}

// MARK: - Private

extension WalletSearchSceneViewModel {
    private var assetsPreviewLimit: Int {
        searchModel.assetsLimit(scope: searchQuery.request.scope)
    }

    private func enableAsset(_ assetId: AssetId) {
        Task {
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
            } catch {
                debugLog("WalletSearchSceneViewModel enable asset error: \(error)")
            }
        }
    }

    private func updateRecent(_ asset: Asset) {
        do {
            try activityService.updateRecent(data: .search(asset), walletId: wallet.id)
        } catch {
            debugLog("UpdateRecent error: \(error)")
        }
    }

    private func updateRequest() {
        if searchModel.searchableQuery.isNotEmpty && searchModel.focus == .tags {
            searchModel.focus = .search
            searchModel.tagsViewModel.selectedTag = AssetTagSelection.all
            searchQuery.request.scope = .all
        }
        searchQuery.request.searchBy = searchModel.searchableQuery
        searchQuery.request.limit = searchModel.fetchLimit(scope: searchQuery.request.scope)
        state = searchModel.searchableQuery.isNotEmpty || !searchQuery.request.scope.isAll ? .loading : .noData
    }

    private func search(query: String, scope: WalletSearchTag = .all) async {
        state = .loading
        do {
            try await searchService.search(wallet: wallet, query: query, scope: scope)
            state = .data(true)
        } catch {
            state.setError(error)
            debugLog("Search error: \(error)")
        }
    }
}
