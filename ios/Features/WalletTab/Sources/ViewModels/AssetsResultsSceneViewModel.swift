// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import protocol Gemstone.GemAssetSelectionServiceProtocol
import struct Gemstone.GemWalletSearchCounts
import struct Gemstone.GemWalletSearchState
import func Gemstone.walletSearchState
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class AssetsResultsSceneViewModel: AssetActions, PerpetualPinActions {
    private let service: any GemAssetSelectionServiceProtocol
    let wallet: Wallet

    let title: String
    let onSelectAssetAction: AssetAction

    public let searchQuery: ObservableQuery<WalletSearchRequest>
    var searchResult: WalletSearchResult {
        searchQuery.value
    }

    var isPresentingToastMessage: ToastMessage?
    private var loadState: StateViewType<Bool> = .loading

    public init(
        wallet: Wallet,
        service: any GemAssetSelectionServiceProtocol,
        request: WalletSearchRequest,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) {
        self.wallet = wallet
        self.service = service
        self.title = title
        var request = request
        request.searchKey = service.searchKey(query: request.searchBy, scope: request.scope.gemScope)
        request.limit = Int(service.walletSearchLimits(query: request.searchBy).results)
        searchQuery = ObservableQuery(request, initialValue: .empty)
        onSelectAssetAction = onSelectAsset
    }

    var currency: Currency {
        service.getCurrency().toPrimitives()
    }

    var sections: WalletSearchSections {
        .from(searchResult, nfts: [])
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var perpetuals: [PerpetualData] {
        searchResult.perpetuals
    }

    private var listsPerpetuals: Bool {
        searchQuery.request.scope.isList && service.showPerpetuals(walletType: wallet.type.toGem(), chains: wallet.chains.map(\.rawValue))
    }

    var state: GemWalletSearchState {
        walletSearchState(
            counts: GemWalletSearchCounts(
                recents: 0,
                pinnedAssets: UInt32(sections.pinnedAssets.count),
                assets: UInt32(sections.assets.count),
                pinnedPerpetuals: 0,
                perpetuals: listsPerpetuals ? UInt32(perpetuals.count) : 0,
                lists: 0,
                nfts: 0,
            ),
            isLoading: loadState.isLoading,
        )
    }

    func searchState(_ state: GemWalletSearchState) -> SearchContentState {
        switch state.phase {
        case .idle: .results
        case .loading: .loading
        case .empty: .empty(EmptyContentType(.searchAssets))
        }
    }

    func contextMenuItems(for assetData: AssetData) -> [ContextMenuItemType] {
        AssetContextMenu.items(
            for: assetData,
            onCopy: { [weak self] in
                self?.isPresentingToastMessage = .copy(
                    CopyTypeViewModel(content: addressCopy(chain: assetData.asset.chain.toGem(), address: $0)).message,
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
    func load() {
        Task { await refresh() }
    }

    func refresh() async {
        loadState = .loading
        do {
            try await service.search(query: searchQuery.request.searchBy, scope: searchQuery.request.scope)
            loadState = .data(true)
        } catch {
            loadState.setError(error)
        }
    }

    func onSelectAsset(_ asset: Asset) {
        onSelectAssetAction?(asset)
        Task { [service] in
            do {
                try await service.addRecent(action: .open, asset: asset.toGem())
            } catch {
                debugLog("AssetsResultsSceneViewModel update recent error: \(error)")
            }
        }
    }
}

extension AssetsResultsSceneViewModel {
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
        ListAssetItemsViewModel(currency: currency, rowStyle: service.flow(selectType: .walletSearchResults).rowStyle)
    }
}
