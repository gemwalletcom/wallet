// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Foundation
import protocol Gemstone.GemAssetSelectionServiceProtocol
import struct Gemstone.GemSearchListRow
import struct Gemstone.GemWalletSearchInput
import struct Gemstone.GemWalletSearchView
import GemstonePrimitives
import GemstoneServices
import Localization
import NFT
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class WalletSearchSceneViewModel: Sendable, SearchResultActions {
    let service: any GemAssetSelectionServiceProtocol

    let wallet: Wallet
    private let onDismissSearch: VoidAction
    private let onAddToken: VoidAction

    private var loadState: StateViewType<Bool> = .noData

    var searchableQuery: String = .empty

    public let searchQuery: ObservableQuery<WalletSearchQuery>
    public let recentModel: RecentAssetsViewModel

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

    var view: GemWalletSearchView {
        let result = searchResult
        return service.walletSearchView(input: GemWalletSearchInput(
            wallet: wallet.toGem(),
            query: searchQuery.request.searchBy,
            isLoading: loadState.isLoading,
            recents: UInt32(recentModel.assets.count),
            assetIds: result.assets.map(\.asset.id),
            pinnedAssetIds: result.assets.filter(\.metadata.isPinned).map(\.asset.id),
            perpetuals: result.perpetuals.map { $0.toGem() },
            lists: result.lists.map { $0.toGem() },
            collections: result.collections.map { $0.toGem() },
        ))
    }

    func searchState(_ view: GemWalletSearchView) -> SearchContentState {
        switch view.state.phase {
        case .idle:
            .results
        case .loading:
            .loading
        case .empty:
            .empty(EmptyStateViewModel(state: view.emptyState) { [weak self] action in
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

    func onSelectRecent(asset: Asset) {
        onSelectAssetAction?(asset)
        recentModel.dismiss()
    }

    func onSelectAddCustomToken() {
        onAddToken?()
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
    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: currency, rowStyle: service.flow(selectType: .walletSearch).rowStyle)
    }
}
