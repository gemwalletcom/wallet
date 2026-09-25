// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNetworkAssetSections
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

    public let activeQuery: ObservableQuery<AssetsRequest>
    public let hiddenQuery: ObservableQuery<AssetsRequest>

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
            AssetsRequest(walletId: wallet.id, filters: [.chains([chain.rawValue]), .enabledBalance], limit: nil),
            initialValue: [],
        )
        hiddenQuery = ObservableQuery(
            AssetsRequest(walletId: wallet.id, filters: [.chains([chain.rawValue]), .disabledBalance, .hasBalance], limit: nil),
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

    var emptyModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.networkAssets, actions: [.manageTokenList: onManageAssetsAction]))
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
    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(assetId: assetId, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: service.getCurrency().toPrimitives(), rowStyle: service.assetRowStyle())
    }
}

struct NetworkAssetGroups {
    let pinned: [AssetData]
    let unpinned: [AssetData]
    let hidden: [AssetData]
    let sections: GemNetworkAssetSections
}
