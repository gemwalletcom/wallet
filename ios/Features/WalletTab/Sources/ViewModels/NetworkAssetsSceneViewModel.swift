// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstoneServices
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class NetworkAssetsSceneViewModel: AssetActions {
    let balanceService: any GemBalanceServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    let wallet: Wallet
    private let onManageAssetsAction: () -> Void

    public var isPresentingToastMessage: ToastMessage?

    public let activeQuery: ObservableQuery<AssetsRequest>
    public let hiddenQuery: ObservableQuery<AssetsRequest>

    public init(
        wallet: Wallet,
        chain: Chain,
        balanceService: any GemBalanceServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        onManageAssets: @escaping () -> Void,
    ) {
        self.wallet = wallet
        self.balanceService = balanceService
        self.preferencesService = preferencesService
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
        preferencesService.currencyCode
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
        pinned.isNotEmpty
    }

    var showUnpinned: Bool {
        unpinned.isNotEmpty
    }

    var showHidden: Bool {
        hidden.isNotEmpty
    }

    var hiddenTitle: String {
        Localized.Common.hidden
    }

    var showEmpty: Bool {
        active.isEmpty && hidden.isEmpty
    }

    var emptyModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .networkAssets(action: onManageAssetsAction))
    }

    var assetIds: [AssetId] {
        (active + hidden).map(\.asset.id)
    }

    func updateBalances() async {
        do {
            try await balanceService.update(walletId: wallet.id.id, assetIds: assetIds.ids)
        } catch {
            debugLog("update balance error: \(error)")
        }
    }

    func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }
}
