// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemTransactionDetailsService
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Transactions

public extension ViewModelFactory {
    @MainActor
    func transactionScene(
        transactionId: TransactionId,
        wallet: Wallet,
        onHeaderAction: @escaping (GemTransactionHeaderAction) -> Void,
        onAddContact: @escaping (AddContactType) -> Void,
        onSelectAddress: @escaping @MainActor @Sendable (ChainAddress) -> Void,
    ) -> TransactionSceneViewModel? {
        guard let transaction = try? stores.transactionStore.getTransaction(walletId: wallet.id, transactionId: transactionId) else {
            return nil
        }
        return TransactionSceneViewModel(
            transaction: transaction,
            wallet: wallet,
            service: Gemstone.GemTransactionDetailsService(explorer: explorerService, preferences: preferencesService),
            onHeaderAction: onHeaderAction,
            onAddContact: onAddContact,
            onSelectAddress: onSelectAddress,
        )
    }

    @MainActor
    func transactionsScene(wallet: Wallet, type: TransactionsRequestType) -> TransactionsViewModel {
        TransactionsViewModel(service: transactionsService, wallet: wallet, type: type)
    }
}
