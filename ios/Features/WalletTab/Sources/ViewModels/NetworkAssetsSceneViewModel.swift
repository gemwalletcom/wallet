// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNetworkAssetIds
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

    var currency: Currency {
        service.getCurrency().toPrimitives()
    }

    var pinned: [AssetData] {
        assets(networkIds.pinned)
    }

    var unpinned: [AssetData] {
        assets(networkIds.unpinned)
    }

    var hidden: [AssetData] {
        assets(networkIds.hidden)
    }

    var showPinned: Bool {
        sections.showsPinned
    }

    var showUnpinned: Bool {
        sections.showsUnpinned
    }

    var showHidden: Bool {
        sections.showsHidden
    }

    var hiddenTitle: String {
        Localized.Common.hidden
    }

    var showEmpty: Bool {
        sections.showsEmpty
    }

    private var sections: GemNetworkAssetSections {
        networkIds.sections
    }

    private var networkIds: GemNetworkAssetIds {
        let active = activeQuery.value
        return networkAssetSections(
            active: active.map(\.asset.id.identifier),
            pinned: active.filter(\.metadata.isPinned).map(\.asset.id.identifier),
            hidden: hiddenQuery.value.map(\.asset.id.identifier),
        )
    }

    private func assets(_ ids: [String]) -> [AssetData] {
        let byId = Dictionary((activeQuery.value + hiddenQuery.value).map { ($0.asset.id.identifier, $0) }, uniquingKeysWith: { first, _ in first })
        return ids.compactMap { byId[$0] }
    }

    var emptyModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.networkAssets, actions: [.manageTokenList: onManageAssetsAction]))
    }

    var assetIds: [AssetId] {
        (pinned + unpinned + hidden).map(\.asset.id)
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
        ListAssetItemsViewModel(currency: currency, rowStyle: service.assetRowStyle())
    }
}
