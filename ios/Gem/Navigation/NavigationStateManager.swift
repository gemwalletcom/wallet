// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
    var pendingWalletNavigation: (walletId: WalletId, path: [any Hashable & Codable])?

    init() {}
}

// MARK: - Business Logic

@MainActor
extension NavigationStateManager {
    func select(tab: TabItem) {
        selectedTab = tab
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

    func reset(walletId: WalletId?) {
        for tabItem in TabItem.allCases {
            backToRoot(tab: tabItem)
        }
        selectedTab = .wallet
        if let pendingWalletNavigation, pendingWalletNavigation.walletId == walletId {
            wallet.setPath(pendingWalletNavigation.path)
        }
        pendingWalletNavigation = nil
    }

    func appendWallet(_ scene: any Hashable & Codable) {
        wallet.append(scene)
        selectedTab = .wallet
        previousSelectedTab = .wallet
    }

    func openWallet(path: [any Hashable & Codable]) {
        wallet.setPath(path)
        selectedTab = .wallet
        previousSelectedTab = .wallet
    }
}
