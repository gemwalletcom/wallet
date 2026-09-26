package com.gemwallet.android.data.services.store.database

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.TransactionsFilter
import com.wallet.core.primitives.WalletId

private fun TransactionsFilter?.toSqlClauses(): List<SqlClause> = if (this == null) {
    emptyList()
} else {
    listOfNotNull(
        assetId?.toIdentifier()?.let { id -> SqlClause.raw("(tx.assetId = ? OR EXISTS (SELECT 1 FROM transactions_assets AS ta WHERE ta.tx_id = tx.id AND ta.asset_id = ?))", id, id) },
        SqlClause.inList("asset.chain", chains.map { it.string }),
        SqlClause.inList("tx.type", transactionTypes.map { it.name }),
        assetRankGreaterThan?.let { SqlClause.greaterThan("asset.rank", it) },
        SqlClause.inList("tx.state", states.map { it.name }),
    )
}

fun buildTransactionListSql(walletId: WalletId, filter: TransactionsFilter?, limit: Int): SqlQuery {
    val source = TRANSACTION_LIST_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT $TRANSACTION_LIST_COLUMNS $source", baseArgs = listOf(walletId.id))
        .whereAll(filter.toSqlClauses())
        .orderBy("tx.createdAt DESC")
        .limit(limit)
        .build()
}

fun buildTransactionsCountSql(walletId: WalletId, filter: TransactionsFilter?): SqlQuery {
    val source = TRANSACTION_LIST_SOURCE.replace(":walletId", "?")
    return SqlQueryBuilder(baseSql = "SELECT COUNT(DISTINCT tx.id) $source", baseArgs = listOf(walletId.id))
        .whereAll(filter.toSqlClauses())
        .build()
}
