package com.gemwallet.android.features.transactions.presents.list

import com.gemwallet.android.ui.components.filters.TransactionFilterUIModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId

internal sealed interface TransactionsListAction {
    data object Refresh : TransactionsListAction
    data object ClearChainsFilter : TransactionsListAction
    data object ClearTypesFilter : TransactionsListAction
    data object Buy : TransactionsListAction
    data object Receive : TransactionsListAction
    data class OpenTransaction(val transactionId: TransactionId) : TransactionsListAction
    data class SelectChainsFilter(val chains: List<Chain>) : TransactionsListAction
    data class SelectTypesFilter(val types: List<TransactionFilterUIModel>) : TransactionsListAction
}
