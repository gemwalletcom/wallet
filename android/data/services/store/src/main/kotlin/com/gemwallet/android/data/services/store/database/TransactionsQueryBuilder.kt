package com.gemwallet.android.data.services.store.database

import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.WalletId

private fun TransactionsQueryFilter.toSqlClause(): SqlClause = when (this) {
    is TransactionsQueryFilter.Chains -> SqlClause.inList("asset.chain", chains.map { it.string })

    is TransactionsQueryFilter.Types -> SqlClause.inList("tx.type", types.map { it.name })

    is TransactionsQueryFilter.AssetRankGreaterThan -> SqlClause.greaterThan("asset.rank", rank)

    is TransactionsQueryFilter.Asset -> {
        val id = assetId.toIdentifier()
        SqlClause.raw("(tx.assetId = ? OR EXISTS (SELECT 1 FROM transactions_assets AS ta WHERE ta.tx_id = tx.id AND ta.asset_id = ?))", id, id)
    }

    is TransactionsQueryFilter.States -> SqlClause.inList("tx.state", states.map { it.name })
}

fun buildTransactionListSql(walletId: WalletId, filters: List<TransactionsQueryFilter>, limit: Int): SqlQuery {
    val source = TRANSACTION_LIST_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT $TRANSACTION_LIST_COLUMNS $source", baseArgs = listOf(walletId.id))
        .whereAll(filters.map { it.toSqlClause() })
        .orderBy("tx.createdAt DESC")
        .limit(limit)
        .build()
}

fun buildTransactionsCountSql(walletId: WalletId, filters: List<TransactionsQueryFilter>): SqlQuery {
    val source = TRANSACTION_LIST_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT COUNT(DISTINCT tx.id) $source", baseArgs = listOf(walletId.id))
        .whereAll(filters.map { it.toSqlClause() })
        .build()
}
