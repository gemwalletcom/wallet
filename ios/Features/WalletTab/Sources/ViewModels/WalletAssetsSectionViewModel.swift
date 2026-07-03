// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Components
import DiscoverAssetsService
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
public final class WalletAssetsSectionViewModel: Sendable {
    private let assetDiscoveryService: any AssetDiscoverable
    private let balanceService: BalanceService

    let observablePreferences: ObservablePreferences

    public let query: ObservableQuery<AssetsRequest>

    public var isPresentingSelectAssetType: SelectAssetType?
    public var isPresentingToastMessage: ToastMessage?

    var wallet: Wallet
    var isLoadingAssets = false

    init(
        assetDiscoveryService: any AssetDiscoverable,
        balanceService: BalanceService,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
    ) {
        self.assetDiscoveryService = assetDiscoveryService
        self.balanceService = balanceService
        self.observablePreferences = observablePreferences
        self.wallet = wallet
        query = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: [.enabledBalance]), initialValue: [])
    }

    var assets: [AssetData] {
        query.value
    }

    var sections: AssetsSections {
        AssetsSections.from(assets)
    }

    var showPinnedSection: Bool {
        !sections.pinned.isEmpty
    }

    var currencyCode: String {
        observablePreferences.preferences.currency
    }

    var manageTokenTitle: String {
        Localized.Wallet.manageTokenList
    }

    var manageImage: Image {
        Images.Actions.manage
    }
}

// MARK: - Business Logic

extension WalletAssetsSectionViewModel {
    func fetch() async {
        await updateWallet(for: wallet)
    }

    func fetchOnce() async {
        await fetchOnce(wallet: wallet)
    }

    func refresh(for newWallet: Wallet) {
        isLoadingAssets = false
        wallet = newWallet
        query.request.walletId = newWallet.id

        Task { await fetchOnce(wallet: newWallet) }
    }

    func onSelectManage() {
        isPresentingSelectAssetType = .manage
    }

    func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.id, assetId: assetId)
        } catch {
            debugLog("WalletAssetsSectionViewModel hide asset error: \(error)")
        }
    }

    func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.id, assetId: asset.id)
            isPresentingToastMessage = .pin(asset.name, pinned: value)
        } catch {
            debugLog("WalletAssetsSectionViewModel pin asset error: \(error)")
        }
    }

    func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }
}

// MARK: - Private

extension WalletAssetsSectionViewModel {
    private func fetchOnce(wallet: Wallet) async {
        let shouldShowLoadingAssets = shouldShowInitialLoadingAssets(for: wallet)

        if shouldShowLoadingAssets {
            isLoadingAssets = true
        }

        await updateWallet(for: wallet)

        if shouldShowLoadingAssets, self.wallet.id == wallet.id {
            isLoadingAssets = false
        }
    }

    private func updateWallet(for wallet: Wallet) async {
        let assetIds = assets.map(\.asset.id)
        async let balance: () = balanceService.updateBalance(for: wallet, assetIds: assetIds)
        async let discovery: () = discoverAssets(wallet: wallet)
        _ = await (balance, discovery)
    }

    private func discoverAssets(wallet: Wallet) async {
        do {
            try await assetDiscoveryService.discoverAssets(wallet: wallet)
        } catch {
            debugLog("WalletAssetsSectionViewModel discoverAssets error: \(error)")
        }
    }

    private func shouldShowInitialLoadingAssets(for wallet: Wallet) -> Bool {
        let preferences = WalletPreferences(walletId: wallet.id)
        return !preferences.completeInitialLoadAssets && preferences.assetsTimestamp == .zero
    }
}
