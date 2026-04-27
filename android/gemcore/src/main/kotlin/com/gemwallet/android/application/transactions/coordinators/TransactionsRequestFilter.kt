package com.gemwallet.android.application.transactions.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType

sealed interface TransactionsRequestFilter {
    data class Chains(val chains: List<Chain>) : TransactionsRequestFilter
    data class Types(val types: List<TransactionType>) : TransactionsRequestFilter
    data class AssetRankGreaterThan(val rank: Int) : TransactionsRequestFilter
    data class Asset(val assetId: AssetId) : TransactionsRequestFilter
    data class State(val state: TransactionState) : TransactionsRequestFilter
}
