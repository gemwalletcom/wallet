// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemAssetSelectionServiceProtocol
import struct Gemstone.GemWalletSearchResultsInput
import struct Gemstone.GemWalletSearchResultsView
import struct Gemstone.GemWalletSearchState
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class AssetsResultsSceneViewModel: SearchResultActions {
    let service: any GemAssetSelectionServiceProtocol
    let wallet: Wallet

    let title: String
    let onSelectAssetAction: AssetAction

    public let searchQuery: ObservableQuery<WalletSearchQuery>

    var isPresentingToastMessage: ToastMessage?
    private var loadState: StateViewType<Bool> = .loading

    public init(
        wallet: Wallet,
        service: any GemAssetSelectionServiceProtocol,
        request: WalletSearchQuery,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) {
        self.wallet = wallet
        self.service = service
        self.title = title
        var request = request
        request.searchKey = request.scope.gemScope.searchKey(query: request.searchBy)
        request.limit = Int(service.walletSearchLimits(query: request.searchBy).results)
        searchQuery = ObservableQuery(request, initialValue: .empty)
        onSelectAssetAction = onSelectAsset
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var view: GemWalletSearchResultsView {
        let result = searchResult
        return service.walletSearchResultsView(input: GemWalletSearchResultsInput(
            wallet: wallet.toGem(),
            scope: searchQuery.request.scope.gemScope,
            isLoading: loadState.isLoading,
            assetIds: result.assets.map(\.asset.id),
            pinnedAssetIds: result.assets.filter(\.metadata.isPinned).map(\.asset.id),
            perpetuals: result.perpetuals.map { $0.toGem() },
        ))
    }

    func searchState(_ state: GemWalletSearchState) -> SearchContentState {
        switch state.phase {
        case .idle: .results
        case .loading: .loading
        case .empty: .empty(EmptyStateViewModel(kind: .searchAssets))
        }
    }
}

// MARK: - Actions

extension AssetsResultsSceneViewModel {
    func load() {
        Task { await refresh() }
    }

    func refresh() async {
        loadState = .loading
        do {
            _ = try await service.search(query: searchQuery.request.searchBy, scope: searchQuery.request.scope.gemScope)
            loadState = .data(true)
        } catch {
            loadState.setError(error)
        }
    }
}

extension AssetsResultsSceneViewModel {
    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: currency, rowStyle: service.flow(selectType: .walletSearchResults).rowStyle)
    }
}
