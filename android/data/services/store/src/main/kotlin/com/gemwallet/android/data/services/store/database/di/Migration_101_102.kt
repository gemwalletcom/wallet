package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_101_102 : Migration(101, 102) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS banners")
        db.execSQL(
            """
                CREATE TABLE banners (
                    id TEXT NOT NULL,
                    wallet_id TEXT,
                    asset_id TEXT,
                    state TEXT NOT NULL,
                    event TEXT NOT NULL,
                    PRIMARY KEY(id),
                    FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """,
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_event ON banners (event)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_wallet_id ON banners (wallet_id)")
    }
}
