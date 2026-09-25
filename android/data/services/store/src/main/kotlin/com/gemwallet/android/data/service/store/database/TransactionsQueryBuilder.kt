package com.gemwallet.android.data.service.store.database

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.WalletId

private fun TransactionsRequestFilter.toSqlClause(): SqlClause = when (this) {
    is TransactionsRequestFilter.Chains -> SqlClause.inList("asset.chain", chains.map { it.string })

    is TransactionsRequestFilter.Types -> SqlClause.inList("tx.type", types.map { it.name })

    is TransactionsRequestFilter.AssetRankGreaterThan -> SqlClause.greaterThan("asset.rank", rank)

    is TransactionsRequestFilter.Asset -> {
        val id = assetId.toIdentifier()
        SqlClause.raw("(tx.assetId = ? OR EXISTS (SELECT 1 FROM transactions_assets AS ta WHERE ta.tx_id = tx.id AND ta.asset_id = ?))", id, id)
    }

    is TransactionsRequestFilter.States -> SqlClause.inList("tx.state", states.map { it.name })
}

fun buildExtendedTransactionsSql(walletId: WalletId, filters: List<TransactionsRequestFilter>, limit: Int): SqlQuery {
    val source = EXTENDED_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT $EXTENDED_COLUMNS $source", baseArgs = listOf(walletId.id))
        .whereAll(filters.map { it.toSqlClause() })
        .orderBy("tx.createdAt DESC")
        .limit(limit)
        .build()
}

fun buildTransactionsCountSql(walletId: WalletId, filters: List<TransactionsRequestFilter>): SqlQuery {
    val source = EXTENDED_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT COUNT(DISTINCT tx.id) $source", baseArgs = listOf(walletId.id))
        .whereAll(filters.map { it.toSqlClause() })
        .build()
}
