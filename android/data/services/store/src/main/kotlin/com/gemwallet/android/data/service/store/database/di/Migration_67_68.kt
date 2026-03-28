package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_67_68 : Migration(67, 68) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS addresses (
                    chain TEXT NOT NULL,
                    address TEXT NOT NULL,
                    wallet_id TEXT,
                    name TEXT NOT NULL,
                    type TEXT,
                    status TEXT NOT NULL DEFAULT 'Unverified',
                    PRIMARY KEY(chain, address)
                )
            """.trimIndent()
        )
        db.execSQL(
            """
                INSERT OR REPLACE INTO addresses (chain, address, wallet_id, name, type, status)
                SELECT a.chain, a.address, a.wallet_id, w.name, 'InternalWallet', 'Verified'
                FROM accounts a
                INNER JOIN wallets w ON a.wallet_id = w.id
            """.trimIndent()
        )
    }
}
