package com.gemwallet.android.data.service.store.database.di

import androidx.sqlite.db.SupportSQLiteDatabase

internal fun SupportSQLiteDatabase.removeChain(chain: String) {
    val assetIdPattern = "${chain}\\_%"
    fun deleteAssetReferences(table: String, column: String) {
        execSQL(
            "DELETE FROM $table WHERE $column = ? OR $column LIKE ? ESCAPE '\\'",
            arrayOf(chain, assetIdPattern),
        )
    }

    execSQL("DELETE FROM wallets_connections WHERE instr(chains, '\"' || ? || '\"') > 0", arrayOf(chain))
    execSQL(
        "DELETE FROM in_app_notifications WHERE " +
            "instr(item, '\"' || ? || '\"') > 0 OR instr(item, '\"' || ? || '_') > 0",
        arrayOf(chain, chain),
    )
    execSQL(
        "UPDATE asset SET associations = '[]' WHERE " +
            "instr(associations, '\"' || ? || '\"') > 0 OR instr(associations, '\"' || ? || '_') > 0",
        arrayOf(chain, chain),
    )
    execSQL("DELETE FROM contacts_addresses WHERE chain = ?", arrayOf(chain))
    execSQL(
        "DELETE FROM tx_swap_metadata WHERE " +
            "from_asset_id = ? OR from_asset_id LIKE ? ESCAPE '\\' OR " +
            "to_asset_id = ? OR to_asset_id LIKE ? ESCAPE '\\' OR " +
            "tx_id IN (SELECT id FROM transactions WHERE " +
            "assetId = ? OR assetId LIKE ? ESCAPE '\\' OR " +
            "feeAssetId = ? OR feeAssetId LIKE ? ESCAPE '\\')",
        arrayOf(chain, assetIdPattern, chain, assetIdPattern, chain, assetIdPattern, chain, assetIdPattern),
    )
    execSQL(
        "DELETE FROM transactions WHERE assetId = ? OR assetId LIKE ? ESCAPE '\\' OR " +
            "feeAssetId = ? OR feeAssetId LIKE ? ESCAPE '\\'",
        arrayOf(chain, assetIdPattern, chain, assetIdPattern),
    )
    execSQL(
        "DELETE FROM search WHERE assetId = ? OR assetId LIKE ? ESCAPE '\\' OR " +
            "perpetualId IN (SELECT id FROM perpetuals WHERE assetId = ? OR assetId LIKE ? ESCAPE '\\')",
        arrayOf(chain, assetIdPattern, chain, assetIdPattern),
    )
    execSQL(
        "DELETE FROM perpetuals_positions WHERE assetId = ? OR assetId LIKE ? ESCAPE '\\' OR " +
            "perpetualId IN (SELECT id FROM perpetuals WHERE assetId = ? OR assetId LIKE ? ESCAPE '\\')",
        arrayOf(chain, assetIdPattern, chain, assetIdPattern),
    )
    deleteAssetReferences("perpetuals", "assetId")
    deleteAssetReferences("stake_delegations", "assetId")
    deleteAssetReferences("stake_validators", "assetId")
    execSQL(
        "DELETE FROM nft_assets_associations WHERE asset_id IN (SELECT id FROM nft_assets WHERE chain = ?)",
        arrayOf(chain),
    )
    execSQL("DELETE FROM nft_assets WHERE chain = ?", arrayOf(chain))
    execSQL("DELETE FROM nft_collections WHERE chain = ?", arrayOf(chain))
    deleteAssetReferences("balances", "asset_id")
    deleteAssetReferences("asset_links", "asset_id")
    deleteAssetReferences("asset_market", "asset_id")
    deleteAssetReferences("fiat_transactions", "assetId")
    deleteAssetReferences("prices", "asset_id")
    deleteAssetReferences("price_alerts", "assetId")
    execSQL(
        "DELETE FROM recent_assets WHERE asset_id = ? OR asset_id LIKE ? ESCAPE '\\' OR " +
            "to_asset_id = ? OR to_asset_id LIKE ? ESCAPE '\\'",
        arrayOf(chain, assetIdPattern, chain, assetIdPattern),
    )
    execSQL(
        "DELETE FROM banners WHERE chain = ? OR asset_id = ? OR asset_id LIKE ? ESCAPE '\\'",
        arrayOf(chain, chain, assetIdPattern),
    )
    execSQL("DELETE FROM nodes WHERE chain = ?", arrayOf(chain))
    execSQL("DELETE FROM addresses WHERE chain = ?", arrayOf(chain))
    execSQL("DELETE FROM accounts WHERE chain = ?", arrayOf(chain))
    execSQL("DELETE FROM asset WHERE chain = ?", arrayOf(chain))
}
