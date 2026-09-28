// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemNavigationTarget
import GemstonePrimitives
import Primitives
import SwiftUI

@Observable
final class NavigationStateManager: Sendable {
    @MainActor
    var wallet = NavigationPathState()
    @MainActor
    var activity = NavigationPathState()
    @MainActor
    var settings = NavigationPathState()

    @MainActor
    var selectedTab: TabItem = .wallet
    @MainActor
    var previousSelectedTab: TabItem = .wallet

    @MainActor
    var walletTabReselected = false

    @MainActor
    var pendingWalletPath: [any Hashable & Codable] = []

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
        }
    }

    func reset() {
        for tabItem in TabItem.allCases {
            backToRoot(tab: tabItem)
        }
        selectedTab = .wallet
        wallet.setPath(pendingWalletPath)
        pendingWalletPath = []
    }

    func openAsset(target: GemNavigationTarget) {
        guard case let .asset(asset, _, isPerpetual) = target else { return }
        wallet.append(Self.assetScene(asset.toPrimitives(), isPerpetual: isPerpetual))
        selectedTab = .wallet
        previousSelectedTab = .wallet
    }

    static func assetScene(_ asset: Asset, isPerpetual: Bool) -> any Hashable & Codable {
        isPerpetual ? Scenes.Perpetual(asset) : Scenes.Asset(asset: asset)
    }

    func openWallet(path: [any Hashable & Codable]) {
        wallet.setPath(path)
        selectedTab = .wallet
        previousSelectedTab = .wallet
    }
}
