// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Foundation
import func Gemstone.addressCopy
import protocol Gemstone.GemAssetSelectionServiceProtocol
import struct Gemstone.GemNftEntry
import struct Gemstone.GemPerpetualMarketItem
import struct Gemstone.GemSearchListRow
import struct Gemstone.GemToast
import struct Gemstone.GemWalletSearchCounts
import struct Gemstone.GemWalletSearchInput
import struct Gemstone.GemWalletSearchView
import GemstonePrimitives
import GemstoneServices
import Localization
import NFT
import Primitives
import PrimitivesComponents
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

    public let searchQuery: ObservableQuery<WalletSearchQuery>
    public let recentModel: RecentAssetsViewModel

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
        recentModel: RecentAssetsViewModel,
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
            WalletSearchQuery(
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

    var derived: WalletSearchDerived {
        let result = searchResult
        let nfts = service.searchCollections(data: result.collections.map { $0.toGem() }, query: searchQuery.request.searchBy)
        let sections = WalletSearchSections.from(result, nfts: nfts)
        let counts = GemWalletSearchCounts(
            recents: UInt32(recentModel.assets.count),
            pinnedAssets: UInt32(sections.pinnedAssets.count),
            assets: UInt32(sections.assets.count),
            pinnedPerpetuals: UInt32(sections.pinnedPerpetuals.count),
            perpetuals: UInt32(sections.perpetuals.count),
            lists: UInt32(sections.lists.count),
            nfts: UInt32(sections.nfts.count),
        )
        let view = service.walletSearchView(input: GemWalletSearchInput(wallet: wallet.toGem(), query: searchableQuery, isLoading: loadState.isLoading, counts: counts))
        return WalletSearchDerived(sections: sections, view: view)
    }

    var currency: Currency {
        service.getCurrency().toPrimitives()
    }

    func searchState(_ derived: WalletSearchDerived) -> SearchContentState {
        switch derived.view.state.phase {
        case .idle:
            .results
        case .loading:
            .loading
        case .empty:
            .empty(EmptyStateViewModel(state: derived.view.emptyState) { [weak self] action in
                switch action {
                case .addCustomToken: self?.onSelectAddCustomToken()
                case .buy, .swap, .receive, .manageTokenList, .clearFilters: break
                }
            })
        }
    }

    var assetsResultsDestination: Scenes.AssetsResults {
        Scenes.AssetsResults(
            searchQuery: searchQuery.request.searchBy,
            scope: searchQuery.request.scope,
        )
    }

    func listDestination(for row: GemSearchListRow) -> Scenes.AssetsResults {
        Scenes.AssetsResults(
            searchQuery: .empty,
            scope: .list(row.list.id),
            title: row.list.name,
        )
    }

    func contextMenuItems(for assetData: AssetData) -> [ContextMenuItemType] {
        AssetContextMenu.items(
            for: assetData,
            onCopy: { [weak self] in
                self?.onSelectCopyAddress(addressCopy(chain: assetData.asset.chain.toGem(), address: $0).copiedMessage)
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
        var request = searchQuery.request
        request.searchBy = searchableQuery
        request.searchKey = service.searchKey(query: searchableQuery, scope: request.scope.gemScope)
        request.limit = Int(service.walletSearchLimits(query: searchableQuery).fetch)
        searchQuery.request = request
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
    func setAssetPinned(_ asset: Asset, pinned: Bool) async throws -> GemToast {
        try await service.setAssetPinned(asset: asset.toGem(), pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds.ids, enabled: enabled)
    }

    func setPerpetualPinned(_ perpetual: Perpetual, pinned: Bool) async throws -> GemToast {
        try await service.setPerpetualPinned(perpetualId: perpetual.id.identifier, name: perpetual.name, pinned: pinned)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: currency, rowStyle: service.flow(selectType: .walletSearch).rowStyle)
    }
}

struct WalletSearchDerived {
    let sections: WalletSearchSections
    let view: GemWalletSearchView
}

extension WalletSearchDerived {
    var previewAssets: [AssetData] {
        sections.assets.prefix(Int(view.limits.assets)).asArray()
    }

    var previewPerpetuals: [GemPerpetualMarketItem] {
        sections.perpetuals.prefix(Int(view.limits.perpetuals)).asArray()
    }

    var previewNFTs: [GemNftEntry] {
        sections.nfts.prefix(Int(view.limits.nfts)).asArray()
    }
}
