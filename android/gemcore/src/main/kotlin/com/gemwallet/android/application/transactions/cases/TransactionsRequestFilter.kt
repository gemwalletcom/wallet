package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemActivityFilters
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.activityFilters

sealed interface TransactionsRequestFilter {
    data class Chains(val chains: List<Chain>) : TransactionsRequestFilter
    data class Types(val types: List<TransactionType>) : TransactionsRequestFilter
    data class AssetRankGreaterThan(val rank: Int) : TransactionsRequestFilter
    data class Asset(val assetId: AssetId) : TransactionsRequestFilter
    data class States(val states: List<TransactionState>) : TransactionsRequestFilter

    companion object {
        fun activity(chains: List<Chain>, filters: List<GemTransactionFilter>): List<TransactionsRequestFilter> =
            activityFilters(chains.map { it.string }, filters).toRequestFilters()

        fun activityDefaults(): List<TransactionsRequestFilter> = activity(emptyList(), emptyList())
    }
}

private fun GemActivityFilters.toRequestFilters(): List<TransactionsRequestFilter> = buildList {
    add(TransactionsRequestFilter.AssetRankGreaterThan(assetRankGreaterThan))
    if (chains.isNotEmpty()) {
        add(TransactionsRequestFilter.Chains(chains.map { it.requireChain() }))
    }
    add(TransactionsRequestFilter.Types(transactionTypes.map { it.toPrimitives() }))
}
