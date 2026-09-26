package com.gemwallet.android.features.transactions.presents.list

import com.gemwallet.android.ui.components.filters.TransactionFilterUIModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId

internal sealed interface TransactionsAction {
    data object Refresh : TransactionsAction
    data object ClearChainsFilter : TransactionsAction
    data object ClearTypesFilter : TransactionsAction
    data object Buy : TransactionsAction
    data object Receive : TransactionsAction
    data class OpenTransaction(val transactionId: TransactionId) : TransactionsAction
    data class SelectChainsFilter(val chains: List<Chain>) : TransactionsAction
    data class SelectTypesFilter(val types: List<TransactionFilterUIModel>) : TransactionsAction
}
