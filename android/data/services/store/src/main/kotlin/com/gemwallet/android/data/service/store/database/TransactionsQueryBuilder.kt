package com.gemwallet.android.data.service.store.database

import androidx.sqlite.db.SimpleSQLiteQuery
import androidx.sqlite.db.SupportSQLiteQuery
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.ext.toIdentifier

const val DEFAULT_TRANSACTIONS_LIMIT = 50

data class ExtendedTransactionsQuery(val sql: String, val args: List<Any>) {
    fun toSupportSQLiteQuery(): SupportSQLiteQuery = SimpleSQLiteQuery(sql, args.toTypedArray())
}

fun buildExtendedTransactionsSql(
    filters: List<TransactionsRequestFilter>,
    limit: Int = DEFAULT_TRANSACTIONS_LIMIT,
): ExtendedTransactionsQuery {
    val clauses = mutableListOf<String>()
    val args = mutableListOf<Any>()
    filters.forEach { filter ->
        when (filter) {
            is TransactionsRequestFilter.Chains -> {
                if (filter.chains.isNotEmpty()) {
                    val placeholders = filter.chains.joinToString(",") { "?" }
                    clauses += "assetId IN (SELECT id FROM asset WHERE chain IN ($placeholders))"
                    args.addAll(filter.chains.map { it.name })
                }
            }
            is TransactionsRequestFilter.Types -> {
                if (filter.types.isNotEmpty()) {
                    val placeholders = filter.types.joinToString(",") { "?" }
                    clauses += "type IN ($placeholders)"
                    args.addAll(filter.types.map { it.name })
                }
            }
            is TransactionsRequestFilter.AssetRankGreaterThan -> {
                clauses += "assetId IN (SELECT id FROM asset WHERE rank > ?)"
                args += filter.rank
            }
            is TransactionsRequestFilter.Asset -> {
                val id = filter.assetId.toIdentifier()
                clauses += "(assetId = ? OR assetIdFrom = ? OR assetIdTo = ?)"
                repeat(3) { args += id }
            }
            is TransactionsRequestFilter.State -> {
                clauses += "state = ?"
                args += filter.state.name
            }
        }
    }
    val where = if (clauses.isEmpty()) "" else " WHERE " + clauses.joinToString(" AND ")
    args += limit
    val sql = "SELECT * FROM extended_txs$where ORDER BY createdAt DESC LIMIT ?"
    return ExtendedTransactionsQuery(sql, args)
}
