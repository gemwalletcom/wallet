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
                    clauses += "asset.chain IN ($placeholders)"
                    args.addAll(filter.chains.map { it.name })
                }
            }
            is TransactionsRequestFilter.Types -> {
                if (filter.types.isNotEmpty()) {
                    val placeholders = filter.types.joinToString(",") { "?" }
                    clauses += "tx.type IN ($placeholders)"
                    args.addAll(filter.types.map { it.name })
                }
            }
            is TransactionsRequestFilter.AssetRankGreaterThan -> {
                clauses += "asset.rank > ?"
                args += filter.rank
            }
            is TransactionsRequestFilter.Asset -> {
                val id = filter.assetId.toIdentifier()
                clauses += "(tx.assetId = ? OR swap.from_asset_id = ? OR swap.to_asset_id = ?)"
                repeat(3) { args += id }
            }
            is TransactionsRequestFilter.State -> {
                clauses += "tx.state = ?"
                args += filter.state.name
            }
        }
    }
    val extra = if (clauses.isEmpty()) "" else " AND " + clauses.joinToString(" AND ")
    args += limit
    val sql = "SELECT $EXTENDED_COLUMNS $EXTENDED_SOURCE$extra ORDER BY tx.createdAt DESC LIMIT ?"
    return ExtendedTransactionsQuery(sql, args)
}
