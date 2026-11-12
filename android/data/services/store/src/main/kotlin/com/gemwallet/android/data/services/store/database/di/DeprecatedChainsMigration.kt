package com.gemwallet.android.data.services.store.database.di

import androidx.sqlite.db.SupportSQLiteDatabase
import com.wallet.core.primitives.WalletType

private data class ForeignKey(val table: String, val column: String, val parentColumn: String)

private val assetColumnsWithoutForeignKey = listOf(
    "prices" to "asset_id",
    "transactions" to "assetId",
    "transactions_assets" to "asset_id",
    "banners" to "asset_id",
    "price_alerts" to "assetId",
    "recent_assets" to "to_asset_id",
)

private val chainColumnsWithoutForeignKey = listOf(
    "contacts_addresses" to "chain",
)

internal fun SupportSQLiteDatabase.removeDeprecatedChains(chains: List<String>) {
    chains.forEach { removeDeprecatedChain(it) }
}

private fun SupportSQLiteDatabase.removeDeprecatedChain(chain: String) {
    deleteCascading("wallets", "type = ? AND id IN (SELECT wallet_id FROM accounts WHERE chain = ?)", arrayOf(WalletType.View.name, chain))
    deleteCascading("accounts", "chain = ? AND wallet_id IN (SELECT id FROM wallets WHERE type = ?)", arrayOf(chain, WalletType.Multicoin.name))
    val assetReferences = assetColumnsWithoutForeignKey.filter { (table, column) -> hasColumn(table, column) } +
        foreignKeys("asset").filter { it.table != "accounts" }.map { it.table to it.column }
    assetReferences.forEach { (table, column) ->
        deleteCascading(table, "`$column` = ? OR `$column` LIKE ? ESCAPE '\\'", arrayOf(chain, "$chain\\_%"))
    }
    chainColumnsWithoutForeignKey.filter { (table, column) -> hasColumn(table, column) }.forEach { (table, column) ->
        deleteCascading(table, "`$column` = ?", arrayOf(chain))
    }
    deleteCascading("asset", "chain = ? AND (id != ? OR id NOT IN (SELECT chain FROM accounts))", arrayOf(chain, chain))
}

private fun SupportSQLiteDatabase.deleteCascading(table: String, condition: String, args: Array<Any>) {
    val references = foreignKeys(table)
    if (references.isEmpty()) {
        execSQL("DELETE FROM `$table` WHERE $condition", args)
        return
    }
    val rowIds = query("SELECT rowid FROM `$table` WHERE $condition", args).use { cursor ->
        buildList { while (cursor.moveToNext()) add(cursor.getLong(0)) }
    }
    if (rowIds.isEmpty()) {
        return
    }
    val selected = "rowid IN (${rowIds.joinToString()})"
    references.forEach { reference ->
        deleteCascading(reference.table, "`${reference.column}` IN (SELECT `${reference.parentColumn}` FROM `$table` WHERE $selected)", emptyArray())
    }
    execSQL("DELETE FROM `$table` WHERE $selected")
}

private fun SupportSQLiteDatabase.foreignKeys(parent: String): List<ForeignKey> = tableNames().flatMap { table ->
    query("PRAGMA foreign_key_list(`$table`)").use { cursor ->
        buildList {
            while (cursor.moveToNext()) {
                if (cursor.getString(cursor.getColumnIndexOrThrow("table")) == parent) {
                    add(ForeignKey(table, cursor.getString(cursor.getColumnIndexOrThrow("from")), cursor.getString(cursor.getColumnIndexOrThrow("to"))))
                }
            }
        }
    }
}

private fun SupportSQLiteDatabase.tableNames(): List<String> = query("SELECT name FROM sqlite_master WHERE type = 'table'").use { cursor ->
    buildList { while (cursor.moveToNext()) add(cursor.getString(0)) }
}

private fun SupportSQLiteDatabase.hasColumn(table: String, column: String): Boolean = query("PRAGMA table_info(`$table`)").use { cursor ->
    generateSequence { if (cursor.moveToNext()) cursor.getString(cursor.getColumnIndexOrThrow("name")) else null }.any { it == column }
}
