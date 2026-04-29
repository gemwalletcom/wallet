// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Preferences
import Primitives
import Store

@Observable
@MainActor
final class MainTabViewModel {
    var wallet: Wallet
    let transactionsQuery: ObservableQuery<TransactionsCountRequest>

    var transactions: Int {
        transactionsQuery.value
    }

    var isPresentingToastMessage: ToastMessage?

    init(wallet: Wallet) {
        self.wallet = wallet
        transactionsQuery = ObservableQuery(TransactionsCountRequest(walletId: wallet.walletId, state: .pending), initialValue: 0)
    }

    var walletId: WalletId {
        wallet.walletId
    }

    func onChangeWallet(_ _: Wallet?, _ newWallet: Wallet?) {
        guard let newWallet else { return }
        wallet = newWallet
        transactionsQuery.request.walletId = newWallet.walletId
    }

    var isMarketEnabled: Bool {
        false // TODO: Disabled. Preferences.standard.isDeveloperEnabled && wallet.type == .multicoin
    }

    var isCollectionsEnabled: Bool {
        switch wallet.type {
        case .multicoin: true
        case .single, .privateKey, .view:
            wallet.accounts.first?.chain.isNFTSupported ?? false
        }
    }
}
