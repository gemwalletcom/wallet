// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Transactions
import class Gemstone.GemTransactionDetailsService
import enum Gemstone.GemTransactionHeaderAction

extension ViewModelFactory {
    @MainActor
    public func transactionScene(
        transaction: TransactionExtended,
        walletId: WalletId,
        onHeaderAction: @escaping (GemTransactionHeaderAction) -> Void,
        onAddContact: @escaping (AddContactType) -> Void,
    ) -> TransactionSceneViewModel {
        TransactionSceneViewModel(
            transaction: transaction,
            walletId: walletId,
            service: Gemstone.GemTransactionDetailsService(explorer: explorerService, preferences: preferencesService),
            onHeaderAction: onHeaderAction,
            onAddContact: onAddContact,
        )
    }

    @MainActor
    public func transactionsScene(wallet: Wallet, type: TransactionsRequestType) -> TransactionsViewModel {
        TransactionsViewModel(service: transactionsService, wallet: wallet, type: type)
    }
}
