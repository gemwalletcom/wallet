package com.gemwallet.android.application.transactions.values

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemActivityFilters
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.activityFilters

sealed interface TransactionsQueryFilter {
    data class Chains(val chains: List<Chain>) : TransactionsQueryFilter
    data class Types(val types: List<TransactionType>) : TransactionsQueryFilter
    data class AssetRankGreaterThan(val rank: Int) : TransactionsQueryFilter
    data class Asset(val assetId: AssetId) : TransactionsQueryFilter
    data class States(val states: List<TransactionState>) : TransactionsQueryFilter

    companion object {
        fun activity(chains: List<Chain>, filters: List<GemTransactionFilter>): List<TransactionsQueryFilter> = activityFilters(chains.map { it.string }, filters).toRequestFilters()

        fun activityDefaults(): List<TransactionsQueryFilter> = activity(emptyList(), emptyList())

        fun pendingActivity(): List<TransactionsQueryFilter> = activityFilters(emptyList(), emptyList()).let { it.toRequestFilters() + States(it.pendingStates.map { state -> state.toPrimitives() }) }
    }
}

private fun GemActivityFilters.toRequestFilters(): List<TransactionsQueryFilter> = buildList {
    add(TransactionsQueryFilter.AssetRankGreaterThan(assetRankGreaterThan))
    if (chains.isNotEmpty()) {
        add(TransactionsQueryFilter.Chains(chains.map { it.requireChain() }))
    }
    add(TransactionsQueryFilter.Types(transactionTypes.map { it.toPrimitives() }))
}
