// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletHomeServiceProtocol
import GemstoneServices
import Components
import Foundation
import Localization
import struct Gemstone.GemAssetRow
import struct Gemstone.GemNetworkAssetCounts
import struct Gemstone.GemNetworkAssetSections
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
            AssetsRequest(walletId: wallet.id, filters: [.chains([chain.rawValue]), .enabledBalance]),
            initialValue: [],
        )
        hiddenQuery = ObservableQuery(
            AssetsRequest(walletId: wallet.id, filters: [.chains([chain.rawValue]), .disabledBalance, .hasBalance]),
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

    var currencyCode: String {
        service.getCurrency()
    }

    var active: [AssetData] {
        activeQuery.value.filter { $0.asset.type != .native }
    }

    var pinned: [AssetData] {
        active.filter { $0.metadata.isPinned }
    }

    var unpinned: [AssetData] {
        active.filter { !$0.metadata.isPinned }
    }

    var hidden: [AssetData] {
        hiddenQuery.value.filter { $0.asset.type != .native }
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
        GemNetworkAssetCounts(
            pinned: UInt32(pinned.count),
            unpinned: UInt32(unpinned.count),
            hidden: UInt32(hidden.count),
        ).sections()
    }

    var emptyModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .networkAssets(action: onManageAssetsAction))
    }

    var assetIds: [AssetId] {
        (active + hidden).map(\.asset.id)
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
    var assetRow: GemAssetRow {
        service.assetRow()
    }

}
