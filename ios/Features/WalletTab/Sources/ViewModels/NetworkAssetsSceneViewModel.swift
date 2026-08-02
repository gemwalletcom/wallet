// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Components
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class NetworkAssetsSceneViewModel {
    private let balanceService: BalanceService
    private let assetsEnabler: any AssetsEnabler
    private let preferences: ObservablePreferences
    private let wallet: Wallet
    private let onManageAssetsAction: () -> Void

    public var isPresentingToastMessage: ToastMessage?

    public let activeQuery: ObservableQuery<AssetsRequest>
    public let hiddenQuery: ObservableQuery<AssetsRequest>

    public init(
        wallet: Wallet,
        chain: Chain,
        balanceService: BalanceService,
        assetsEnabler: any AssetsEnabler,
        preferences: ObservablePreferences = .default,
        onManageAssets: @escaping () -> Void,
    ) {
        self.wallet = wallet
        self.balanceService = balanceService
        self.assetsEnabler = assetsEnabler
        self.preferences = preferences
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
        preferences.preferences.currency
    }

    var hideBalance: Bool {
        preferences.isHideBalanceEnabled
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
        await balanceService.updateBalance(for: wallet, assetIds: assetIds)
    }

    func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.id, assetId: assetId)
        } catch {
            debugLog("NetworkAssetsSceneViewModel hide asset error: \(error)")
        }
    }

    func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.id, assetId: asset.id)
            isPresentingToastMessage = .pin(asset.name, pinned: value)
        } catch {
            debugLog("NetworkAssetsSceneViewModel pin asset error: \(error)")
        }
    }

    func onAddToWallet(_ assetId: AssetId) {
        Task {
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
                isPresentingToastMessage = .addedToWallet()
            } catch {
                debugLog("NetworkAssetsSceneViewModel add asset error: \(error)")
            }
        }
    }

    func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }
}
