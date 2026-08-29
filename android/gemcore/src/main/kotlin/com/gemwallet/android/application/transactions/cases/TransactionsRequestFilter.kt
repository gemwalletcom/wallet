package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemAssetConfigService

private val assetConfig = GemAssetConfigService()

sealed interface TransactionsRequestFilter {
    data class Chains(val chains: List<Chain>) : TransactionsRequestFilter
    data class Types(val types: List<TransactionType>) : TransactionsRequestFilter
    data class AssetRankGreaterThan(val rank: Int) : TransactionsRequestFilter
    data class Asset(val assetId: AssetId) : TransactionsRequestFilter
    data class States(val states: List<TransactionState>) : TransactionsRequestFilter

    companion object {
        fun activityDefaults(): List<TransactionsRequestFilter> = listOf(
            AssetRankGreaterThan(assetConfig.defaultTokenRank()),
        )
    }
}
