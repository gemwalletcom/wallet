// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemEmptyState
import struct Gemstone.GemNetworkAssetSections
import struct Gemstone.GemToast
import protocol Gemstone.GemWalletHomeServiceProtocol
import func Gemstone.networkAssetSections
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class NetworkAssetsSceneViewModel: AssetActions {
    private let service: any GemWalletHomeServiceProtocol
    private let onManageAssetsAction: () -> Void

    public var isPresentingToastMessage: ToastMessage?

    public let activeQuery: ObservableQuery<AssetsQuery>
    public let hiddenQuery: ObservableQuery<AssetsQuery>

    public init(
        wallet: Wallet,
        chain: Chain,
        service: any GemWalletHomeServiceProtocol,
        onManageAssets: @escaping () -> Void,
    ) {
        self.service = service
        onManageAssetsAction = onManageAssets
        activeQuery = ObservableQuery(
            AssetsQuery(walletId: wallet.id, filters: [.chains([chain.rawValue]), .enabledBalance], limit: nil),
            initialValue: [],
        )
        hiddenQuery = ObservableQuery(
            AssetsQuery(walletId: wallet.id, filters: [.chains([chain.rawValue]), .disabledBalance, .hasBalance], limit: nil),
            initialValue: [],
        )
    }

    var title: String {
        Localized.Assets.title
    }

    var manageImage: Image {
        Images.Actions.manage
    }

    func onSelectManageAssets() {
        onManageAssetsAction()
    }

    var hiddenTitle: String {
        Localized.Common.hidden
    }

    var groups: NetworkAssetGroups {
        let active = activeQuery.value
        let ids = networkAssetSections(
            active: active.map(\.asset.id),
            pinned: active.filter(\.metadata.isPinned).map(\.asset.id),
            hidden: hiddenQuery.value.map(\.asset.id),
        )
        let rows = active + hiddenQuery.value
        return NetworkAssetGroups(pinned: rows.assets(ids: ids.pinned), unpinned: rows.assets(ids: ids.unpinned), hidden: rows.assets(ids: ids.hidden), sections: ids.sections)
    }

    func emptyModel(_ state: GemEmptyState) -> EmptyStateViewModel {
        EmptyStateViewModel(state: state) { [onManageAssetsAction] action in
            switch action {
            case .manageTokenList: onManageAssetsAction()
            case .buy, .swap, .receive, .addCustomToken, .clearFilters: break
            }
        }
    }

    var assetIds: [AssetId] {
        let groups = groups
        return (groups.pinned + groups.unpinned + groups.hidden).map(\.asset.id)
    }

    func updateBalances() async {
        do {
            try await service.updateBalances(assetIds: assetIds)
        } catch {
            debugLog("update balance error: \(error)")
        }
    }

    func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }
}

extension NetworkAssetsSceneViewModel {
    func setAssetPinned(_ asset: Asset, pinned: Bool) async throws -> GemToast {
        try await service.setAssetPinned(asset: asset.toGem(), pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: service.getCurrency().toPrimitives())
    }
}

struct NetworkAssetGroups {
    let pinned: [AssetData]
    let unpinned: [AssetData]
    let hidden: [AssetData]
    let sections: GemNetworkAssetSections
}
