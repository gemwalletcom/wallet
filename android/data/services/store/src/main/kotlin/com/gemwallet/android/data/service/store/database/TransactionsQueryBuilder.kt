package com.gemwallet.android.data.service.store.database

import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.ext.toIdentifier

const val DEFAULT_TRANSACTIONS_LIMIT = 50

private fun TransactionsRequestFilter.toSqlClause(): SqlClause = when (this) {
    is TransactionsRequestFilter.Chains -> SqlClause.inList("asset.chain", chains.map { it.string })
    is TransactionsRequestFilter.Types -> SqlClause.inList("tx.type", types.map { it.name })
    is TransactionsRequestFilter.AssetRankGreaterThan -> SqlClause.greaterThan("asset.rank", rank)
    is TransactionsRequestFilter.Asset -> {
        val id = assetId.toIdentifier()
        SqlClause.raw("(tx.assetId = ? OR swap.from_asset_id = ? OR swap.to_asset_id = ?)", id, id, id)
    }
    is TransactionsRequestFilter.State -> SqlClause.equalTo("tx.state", state.name)
}

fun buildExtendedTransactionsSql(
    walletId: String,
    filters: List<TransactionsRequestFilter>,
    limit: Int = DEFAULT_TRANSACTIONS_LIMIT,
): SqlQuery {
    val source = EXTENDED_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT $EXTENDED_COLUMNS $source", baseArgs = listOf(walletId))
        .whereAll(filters.map { it.toSqlClause() })
        .orderBy("tx.createdAt DESC")
        .limit(limit)
        .build()
}
