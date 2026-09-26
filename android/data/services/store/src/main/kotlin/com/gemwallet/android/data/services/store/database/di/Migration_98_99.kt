package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_98_99 : Migration(98, 99) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("CREATE TABLE IF NOT EXISTS transactions_assets (tx_id TEXT NOT NULL, asset_id TEXT NOT NULL, PRIMARY KEY(tx_id, asset_id))")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_transactions_assets_asset_id ON transactions_assets (asset_id)")
        db.execSQL("INSERT OR IGNORE INTO transactions_assets (tx_id, asset_id) SELECT tx_id, from_asset_id FROM tx_swap_metadata")
        db.execSQL("INSERT OR IGNORE INTO transactions_assets (tx_id, asset_id) SELECT tx_id, to_asset_id FROM tx_swap_metadata")
        db.execSQL("INSERT OR IGNORE INTO transactions_assets (tx_id, asset_id) SELECT id, assetId FROM transactions WHERE id NOT IN (SELECT tx_id FROM tx_swap_metadata)")
        db.execSQL("DROP TABLE IF EXISTS tx_swap_metadata")
    }
}
