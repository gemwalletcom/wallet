// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
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
    let wallet: Wallet
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
        self.wallet = wallet
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
            active: active.map(\.asset.id.identifier),
            pinned: active.filter(\.metadata.isPinned).map(\.asset.id.identifier),
            hidden: hiddenQuery.value.map(\.asset.id.identifier),
        )
        let byId = Dictionary((active + hiddenQuery.value).map { ($0.asset.id.identifier, $0) }, uniquingKeysWith: { first, _ in first })
        let assets = { (assetIds: [String]) in assetIds.compactMap { byId[$0] } }
        return NetworkAssetGroups(pinned: assets(ids.pinned), unpinned: assets(ids.unpinned), hidden: assets(ids.hidden), sections: ids.sections)
    }

    var emptyModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .networkAssets) { [onManageAssetsAction] action in
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
            try await service.updateBalances(assetIds: assetIds.ids)
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
        try await service.setAssetsEnabled(assetIds: assetIds.ids, enabled: enabled)
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
