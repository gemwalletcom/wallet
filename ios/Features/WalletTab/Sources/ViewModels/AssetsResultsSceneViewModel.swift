// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemAssetSelectionServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Foundation
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
    private var state: StateViewType<Bool> = .loading

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
        request.limit = Int(service.walletSearchLimits(query: request.searchBy).results)
        searchQuery = ObservableQuery(request, initialValue: .empty)
        onSelectAssetAction = onSelectAsset
    }

    var currencyCode: String {
        service.getCurrency()
    }

    var sections: WalletSearchSections {
        .from(searchResult, nfts: [])
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
        searchQuery.request.scope.isList && sections.perpetuals.isNotEmpty && service.showPerpetuals(wallet: wallet.json())
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
    func load() {
        Task { await refresh() }
    }

    func refresh() async {
        state = .loading
        do {
            try await service.search(query: searchQuery.request.searchBy, scope: searchQuery.request.scope)
            state = .data(true)
        } catch {
            state.setError(error)
        }
    }

    func onSelectAsset(_ asset: Asset) {
        onSelectAssetAction?(asset)
        Task { [service] in
            do {
                try await service.addRecent(action: .open, asset: asset.map())
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
}
