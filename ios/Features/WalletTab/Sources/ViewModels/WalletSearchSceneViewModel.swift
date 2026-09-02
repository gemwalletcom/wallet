// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemAssetSelectionServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Foundation
import Localization
import NFT
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class WalletSearchSceneViewModel: Sendable, AssetActions, PerpetualPinActions {
    private let service: any GemAssetSelectionServiceProtocol

    let wallet: Wallet
    private let onDismissSearch: VoidAction
    private let onAddToken: VoidAction

    private var state: StateViewType<Bool> = .noData

    var searchModel: WalletSearchModel

    public let searchQuery: ObservableQuery<WalletSearchRequest>
    public let recentModel: RecentAssetsModel

    var searchResult: WalletSearchResult {
        searchQuery.value
    }

    var isPresentingToastMessage: ToastMessage?
    var isSearching: Bool = false
    var isSearchPresented: Bool = false
    var dismissSearch: Bool = false

    let onSelectAssetAction: AssetAction

    public init(
        wallet: Wallet,
        service: any GemAssetSelectionServiceProtocol,
        recentModel: RecentAssetsModel,
        onDismissSearch: VoidAction,
        onSelectAssetAction: AssetAction,
        onAddToken: VoidAction,
    ) {
        self.wallet = wallet
        self.service = service
        self.recentModel = recentModel
        self.onDismissSearch = onDismissSearch
        self.onSelectAssetAction = onSelectAssetAction
        self.onAddToken = onAddToken
        searchModel = WalletSearchModel()

        searchQuery = ObservableQuery(
            WalletSearchRequest(
                walletId: wallet.id,
                limit: WalletSearchModel.initialFetchLimit,
                types: WalletSearchModel.searchItemTypes,
            ),
            initialValue: .empty,
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

    var collectionsTitle: String {
        Localized.Nft.collections
    }

    var collectionsContent: CollectionsContent {
        CollectionsContent(items: previewNFTs.map { item in
            switch item {
            case let .collection(data): NFTGridPosterBuilder.item(from: data)
            case let .asset(assetData): NFTGridPosterBuilder.item(collection: assetData.collection, asset: assetData.asset)
            }
        })
    }

    var sections: WalletSearchSections {
        .from(searchResult)
    }

    var currencyCode: String {
        service.currency()
    }

    var showRecents: Bool {
        searchModel.searchableQuery.isEmpty && recentModel.hasAssets
    }

    var showPerpetuals: Bool {
        sections.perpetuals.isNotEmpty && service.showPerpetuals(wallet: wallet.json())
    }

    var searchState: SearchContentState {
        guard showEmpty else { return .results }
        if state.isLoading { return .loading }
        return .empty(.search(
            type: .assets,
            action: showAddToken ? { [weak self] in self?.onSelectAddCustomToken() } : nil,
        ))
    }

    var showEmpty: Bool {
        !showRecents && !showPinned && !showAssets && !showPerpetuals && !showLists && !showNFTs
    }

    var showPinned: Bool {
        sections.pinnedAssets.isNotEmpty || showPinnedPerpetuals
    }

    var showPinnedPerpetuals: Bool {
        sections.pinnedPerpetuals.isNotEmpty && service.showPerpetuals(wallet: wallet.json())
    }

    var showAssets: Bool {
        sections.assets.isNotEmpty
    }

    var showLists: Bool {
        sections.lists.isNotEmpty
    }

    var showNFTs: Bool {
        sections.nfts.isNotEmpty
    }

    var showAddToken: Bool {
        wallet.hasTokenSupport
    }

    var previewAssets: [AssetData] {
        sections.assets.prefix(searchModel.assetsLimit).asArray()
    }

    var previewPerpetuals: [PerpetualData] {
        sections.perpetuals.prefix(searchModel.perpetualsLimit).asArray()
    }

    var previewNFTs: [NFTSearchItem] {
        sections.nfts.prefix(searchModel.nftsLimit).asArray()
    }

    var hasMoreAssets: Bool {
        searchResult.assets.count > searchModel.assetsLimit
    }

    var hasMorePerpetuals: Bool {
        searchResult.perpetuals.count > searchModel.perpetualsLimit
    }

    var hasMoreNFTs: Bool {
        searchResult.nfts.count > searchModel.nftsLimit
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
                self?.onPinAsset(assetData.asset, value: !assetData.metadata.isPinned)
            },
            onAddToWallet: { [weak self] in
                self?.onAddToWallet(assetData.asset.id)
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

    func load() {
        updateRequest()
        Task {
            await search(query: .empty)
        }
    }

    func onSelectAsset(_ asset: Asset) {
        onSelectAssetAction?(asset)
        updateRecent(asset)
    }

    func onSelectRecent(asset: Asset) {
        onSelectAssetAction?(asset)
        recentModel.dismiss()
    }

    func onSelectAddCustomToken() {
        onAddToken?()
    }

    func onSelectCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }

    func onChangeSearchQuery(_: String, _: String) {
        updateRequest()
    }

    func onChangeSearchPresented(_: Bool, isPresented: Bool) {
        guard !isPresented else { return }
        dismissSearch = true
        onDismissSearch?()
    }
}

// MARK: - Private

extension WalletSearchSceneViewModel {
    private func updateRecent(_ asset: Asset) {
        Task { [service] in
            do {
                try await service.addRecent(action: .open, asset: asset.map())
            } catch {
                debugLog("UpdateRecent error: \(error)")
            }
        }
    }

    private func updateRequest() {
        searchQuery.request.searchBy = searchModel.searchableQuery
        searchQuery.request.limit = searchModel.fetchLimit
        state = searchModel.searchableQuery.isNotEmpty ? .loading : .noData
    }

    private func search(query: String) async {
        state = .loading
        do {
            try await service.search(wallet: wallet, query: query, scope: .all)
            state = .data(true)
        } catch {
            state.setError(error)
            debugLog("Search error: \(error)")
        }
    }
}

extension WalletSearchSceneViewModel {
    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(walletId: wallet.id.id, assetId: assetId.identifier, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(walletId: wallet.id.id, assetIds: assetIds.ids, enabled: enabled)
    }

    func setPerpetualPinned(_ perpetualId: PerpetualId, pinned: Bool) async throws {
        try await service.setPerpetualPinned(perpetualId: perpetualId.identifier, pinned: pinned)
    }
}
