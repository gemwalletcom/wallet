package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_85_86 : Migration(85, 86) {
    internal const val SEI = "sei"

    override fun migrate(db: SupportSQLiteDatabase) {
        removeChains(db, setOf(SEI))
    }
}

internal fun removeChains(db: SupportSQLiteDatabase, chains: Set<String>) {
    for (chain in chains) {
        chainRemovalStatements(chain).forEach { statement ->
            db.execSQL(statement.sql, statement.arguments.toTypedArray())
        }
    }
}

internal data class MigrationStatement(
    val sql: String,
    val arguments: List<Any?>,
) {
    override fun toString(): String = "$sql | ${arguments.joinToString()}"
}

internal fun chainRemovalStatements(chain: String): List<MigrationStatement> {
    val assetIdPattern = "${chain}\\_%"
    val quotedChain = "\"$chain\""
    val quotedAssetIdPrefix = "\"${chain}_"
    fun assetId(column: String) = "$column = ? OR $column LIKE ? ESCAPE '\\'"

    return listOf(
        MigrationStatement(
            "DELETE FROM wallets_connections WHERE instr(chains, ?) > 0",
            listOf(quotedChain),
        ),
        MigrationStatement(
            "DELETE FROM in_app_notifications WHERE instr(item, ?) > 0 OR instr(item, ?) > 0",
            listOf(quotedChain, quotedAssetIdPrefix),
        ),
        MigrationStatement(
            "UPDATE asset SET associations = '[]' WHERE instr(associations, ?) > 0 OR instr(associations, ?) > 0",
            listOf(quotedChain, quotedAssetIdPrefix),
        ),
        MigrationStatement("DELETE FROM contacts_addresses WHERE chain = ?", listOf(chain)),
        MigrationStatement(
            "DELETE FROM tx_swap_metadata WHERE ${assetId("from_asset_id")} OR ${assetId("to_asset_id")}",
            listOf(chain, assetIdPattern, chain, assetIdPattern),
        ),
        MigrationStatement(
            "DELETE FROM transactions WHERE ${assetId("assetId")} OR ${assetId("feeAssetId")}",
            listOf(chain, assetIdPattern, chain, assetIdPattern),
        ),
        MigrationStatement(
            "DELETE FROM prices WHERE ${assetId("asset_id")}",
            listOf(chain, assetIdPattern),
        ),
        MigrationStatement(
            "DELETE FROM price_alerts WHERE ${assetId("assetId")}",
            listOf(chain, assetIdPattern),
        ),
        MigrationStatement(
            "DELETE FROM recent_assets WHERE ${assetId("asset_id")} OR ${assetId("to_asset_id")}",
            listOf(chain, assetIdPattern, chain, assetIdPattern),
        ),
        MigrationStatement(
            "DELETE FROM banners WHERE chain = ? OR ${assetId("asset_id")}",
            listOf(chain, chain, assetIdPattern),
        ),
    ) + listOf("nft_assets", "nft_collections", "nodes", "addresses", "accounts", "asset").map { table ->
        MigrationStatement("DELETE FROM $table WHERE chain = ?", listOf(chain))
    }
}
