package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_88_89 : Migration(88, 89) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE banners")
        db.execSQL(
            """
            CREATE TABLE banners (
                id TEXT NOT NULL,
                wallet_id TEXT,
                asset_id TEXT,
                state TEXT NOT NULL,
                event TEXT NOT NULL,
                PRIMARY KEY(id)
            )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_event ON banners(event)")
        db.execSQL("CREATE INDEX IF NOT EXISTS index_banners_wallet_id ON banners(wallet_id)")
    }
}
