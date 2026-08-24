// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Primitives
import SwiftUI

enum AssetNavigationPolicy {
    static func canOpen(_ assetId: AssetId) -> Bool {
        switch assetId.type {
        case .native: assetId.chain.hasNativeAsset
        case .token: true
        }
    }
}

@Observable
final class NavigationStateManager: Sendable {
    @MainActor
    var wallet = NavigationPathState()
    @MainActor
    var activity = NavigationPathState()
    @MainActor
    var settings = NavigationPathState()
    @MainActor
    var markets = NavigationPathState()

    @MainActor
    var selectedTab: TabItem = .wallet
    @MainActor
    var previousSelectedTab: TabItem = .wallet

    @MainActor
    var walletTabReselected = false

    init() {}
}

// MARK: - Business Logic

@MainActor
extension NavigationStateManager {
    func select(tab: TabItem) {
        selectedTab = tab
        // back to root if selected same tab, if some routes already in stack
        guard tab != previousSelectedTab else {
            backToRoot(tab: tab)
            return
        }
        previousSelectedTab = selectedTab
    }

    func backToRoot(tab: TabItem) {
        if wallet.isEmpty {
            walletTabReselected.toggle()
        }

        switch tab {
        case .wallet: wallet.reset()
        case .activity: activity.reset()
        case .settings: settings.reset()
        case .markets: markets.reset()
        }
    }

    func clearAll() {
        for tabItem in TabItem.allCases {
            backToRoot(tab: tabItem)
        }
    }

    func openAsset(_ asset: Asset) {
        guard AssetNavigationPolicy.canOpen(asset.id) else { return }
        if asset.type == .perpetual {
            wallet.append(Scenes.Perpetual(asset))
        } else {
            wallet.append(Scenes.Asset(asset: asset))
        }
        selectedTab = .wallet
        previousSelectedTab = .wallet
    }
}
