// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import protocol Gemstone.GemAssetSelectionServiceProtocol
import enum Gemstone.GemImage
import enum Gemstone.GemNftItem
import struct Gemstone.GemWalletSearchCounts
import struct Gemstone.GemWalletSearchLimits
import struct Gemstone.GemWalletSearchState
import func Gemstone.walletSearchState
import GemstonePrimitives
import GemstoneServices
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

    private var loadState: StateViewType<Bool> = .noData

    var searchableQuery: String = .empty

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

        searchQuery = ObservableQuery(
            WalletSearchRequest(
                walletId: wallet.id,
                limit: Int(service.walletSearchLimits(query: .empty).fetch),
                types: [.asset, .perpetual, .list, .nft],
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
        CollectionsContent(items: NFTGridPosterBuilder.items(previewNFTs))
    }

    var sections: WalletSearchSections {
        .from(searchResult, nfts: nftSearchItems)
    }

    private var nftSearchItems: [GemNftItem] {
        service.searchCollections(data: searchResult.collections.map { $0.toGem() }, query: searchQuery.request.searchBy)
    }

    var searchDebounce: Duration {
        .milliseconds(service.searchDebounceMilliseconds())
    }

    var currency: Currency {
        service.getCurrency().toPrimitives()
    }

    var searchState: SearchContentState {
        switch state.phase {
        case .results:
            .results
        case .loading:
            .loading
        case .empty:
            .empty(.search(
                type: .assets,
                action: showAddToken ? { [weak self] in self?.onSelectAddCustomToken() } : nil,
            ))
        }
    }

    private var state: GemWalletSearchState {
        walletSearchState(
            counts: GemWalletSearchCounts(
                recents: showsRecents ? UInt32(recentModel.assets.count) : 0,
                pinnedAssets: UInt32(sections.pinnedAssets.count),
                assets: UInt32(sections.assets.count),
                pinnedPerpetuals: showsPerpetuals ? UInt32(sections.pinnedPerpetuals.count) : 0,
                perpetuals: showsPerpetuals ? UInt32(sections.perpetuals.count) : 0,
                lists: UInt32(sections.lists.count),
                nfts: UInt32(sections.nfts.count),
            ),
            isLoading: loadState.isLoading,
        )
    }

    private var showsRecents: Bool {
        service.flow(selectType: .walletSearch).showsRecents(isSearching: searchableQuery.isNotEmpty, hasRecents: recentModel.hasAssets)
    }

    private var showsPerpetuals: Bool {
        service.showPerpetuals(walletType: wallet.type.toGem(), chains: wallet.chains.map(\.rawValue))
    }

    var showRecents: Bool {
        state.showsRecents
    }

    var showPerpetuals: Bool {
        state.showsPerpetuals
    }

    var showPinned: Bool {
        state.showsPinned
    }

    var showPinnedPerpetuals: Bool {
        state.showsPinnedPerpetuals
    }

    var showAssets: Bool {
        state.showsAssets
    }

    var showLists: Bool {
        state.showsLists
    }

    var showNFTs: Bool {
        state.showsNfts
    }

    var showAddToken: Bool {
        service.flow(selectType: .walletSearch).showsAddToken(
            supportsTokens: service.supportsTokens(wallet: wallet.toGem()),
            hasChains: service.filterChains(wallet: wallet.toGem()).isNotEmpty,
        )
    }

    private var limits: GemWalletSearchLimits {
        service.walletSearchLimits(query: searchableQuery)
    }

    var previewAssets: [AssetData] {
        sections.assets.prefix(Int(limits.assets)).asArray()
    }

    var previewPerpetuals: [PerpetualData] {
        sections.perpetuals.prefix(Int(limits.perpetuals)).asArray()
    }

    var previewNFTs: [GemNftItem] {
        sections.nfts.prefix(Int(limits.nfts)).asArray()
    }

    var hasMoreAssets: Bool {
        limits.hasMoreAssets(count: UInt32(sections.assets.count))
    }

    var hasMorePerpetuals: Bool {
        limits.hasMorePerpetuals(count: UInt32(sections.perpetuals.count))
    }

    var hasMoreNFTs: Bool {
        limits.hasMoreNfts(count: UInt32(sections.nfts.count))
    }

    var assetsResultsDestination: Scenes.AssetsResults {
        Scenes.AssetsResults(
            searchQuery: searchQuery.request.searchBy,
            scope: searchQuery.request.scope,
        )
    }

    func listItem(for list: AssetList) -> ListItemModel {
        ListItemModel(
            title: list.name,
            subtitle: String(list.count),
            imageStyle: .settings(assetImage: AssetImage(type: .text(list.name), imageURL: GemImage.assetList(listId: list.id).imageURL)),
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
                self?.onSelectCopyAddress(CopyTypeViewModel(content: addressCopy(chain: assetData.asset.chain.toGem(), address: $0)).message)
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
                try await service.addRecent(action: .open, asset: asset.toGem())
            } catch {
                debugLog("UpdateRecent error: \(error)")
            }
        }
    }

    private func updateRequest() {
        searchQuery.request.searchBy = searchableQuery
        searchQuery.request.searchKey = service.searchKey(query: searchableQuery, scope: searchQuery.request.scope.gemScope)
        searchQuery.request.limit = Int(limits.fetch)
        loadState = searchableQuery.isNotEmpty ? .loading : .noData
    }

    private func search(query: String) async {
        loadState = .loading
        do {
            _ = try await service.search(query: query, scope: .all)
            guard query == searchableQuery.trim() else { return }
            loadState = .data(true)
        } catch {
            guard query == searchableQuery.trim() else { return }
            loadState.setError(error)
            debugLog("Search error: \(error)")
        }
    }
}

extension WalletSearchSceneViewModel {
    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(assetId: assetId.identifier, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds.ids, enabled: enabled)
    }

    func setPerpetualPinned(_ perpetualId: PerpetualId, pinned: Bool) async throws {
        try await service.setPerpetualPinned(perpetualId: perpetualId.identifier, pinned: pinned)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: currency, rowStyle: service.flow(selectType: .walletSearch).rowStyle)
    }
}
