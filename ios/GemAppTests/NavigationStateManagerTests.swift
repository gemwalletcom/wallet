// Copyright (c). Gem Wallet. All rights reserved.

@testable import Gem
import Primitives
import SwiftUI
import Testing

@MainActor
struct NavigationStateManagerTests {
    @Test
    func walletSelectionSupersedesTransactionNavigation() {
        let state = NavigationStateManager()
        let notifiedWallet = WalletId.multicoin(address: "wallet3")
        let selectedWallet = WalletId.multicoin(address: "wallet5")
        let transaction = Scenes.Transaction(id: TransactionId(chain: .ethereum, hash: "transaction3"))
        state.pendingWalletNavigation = (notifiedWallet, [transaction])
        state.activity.append("old-transaction")
        state.settings.append("old-settings")

        state.reset(walletId: selectedWallet)

        #expect(state.wallet.isEmpty)
        #expect(state.activity.isEmpty)
        #expect(state.settings.isEmpty)
        #expect(state.pendingWalletNavigation == nil)
        #expect(state.selectedTab == .wallet)

        state.reset(walletId: notifiedWallet)

        #expect(state.wallet.isEmpty)
    }

    @Test
    func transactionNavigationOpensItsWallet() {
        let state = NavigationStateManager()
        let walletId = WalletId.multicoin(address: "wallet3")
        let transaction = Scenes.Transaction(id: TransactionId(chain: .ethereum, hash: "transaction3"))
        state.pendingWalletNavigation = (walletId, [transaction])

        state.reset(walletId: walletId)

        #expect(state.wallet.path == NavigationPath([transaction]))
        #expect(state.pendingWalletNavigation == nil)
    }

    @Test
    func missingWalletDiscardsTransactionNavigation() {
        let state = NavigationStateManager()
        let transaction = Scenes.Transaction(id: TransactionId(chain: .ethereum, hash: "transaction3"))
        state.pendingWalletNavigation = (.multicoin(address: "wallet3"), [transaction])

        state.reset(walletId: nil)

        #expect(state.wallet.isEmpty)
        #expect(state.pendingWalletNavigation == nil)
    }
}
