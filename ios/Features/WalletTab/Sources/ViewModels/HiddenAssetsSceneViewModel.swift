// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class HiddenAssetsSceneViewModel: Sendable {
    private let balanceService: BalanceService

    let observablePreferences: ObservablePreferences

    public var wallet: Wallet

    public let assetsQuery: ObservableQuery<AssetsRequest>

    public init(
        balanceService: BalanceService,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
    ) {
        self.balanceService = balanceService
        self.observablePreferences = observablePreferences
        self.wallet = wallet
        assetsQuery = ObservableQuery(
            AssetsRequest(walletId: wallet.id, filters: [.hiddenBalance, .hasBalance, .assetRank(lessThanOrEqualTo: AssetScore.defaultScore)]),
            initialValue: [],
        )
    }

    var title: String {
        Localized.Asset.Verification.unverified
    }

    var assets: [AssetData] {
        assetsQuery.value
    }

    var currencyCode: String {
        observablePreferences.preferences.currency
    }

    func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.id, assetId: assetId)
        } catch {
            debugLog("HiddenAssetsSceneViewModel hide asset error: \(error)")
        }
    }

    func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.id, assetId: asset.id)
        } catch {
            debugLog("HiddenAssetsSceneViewModel pin asset error: \(error)")
        }
    }
}
