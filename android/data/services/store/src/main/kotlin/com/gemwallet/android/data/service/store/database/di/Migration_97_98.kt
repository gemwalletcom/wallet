package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_96_98 : Migration(96, 98) {
    override fun migrate(db: SupportSQLiteDatabase) = recreateAssetMarket(db)
}

object Migration_97_98 : Migration(97, 98) {
    override fun migrate(db: SupportSQLiteDatabase) = recreateAssetMarket(db)
}

private fun recreateAssetMarket(db: SupportSQLiteDatabase) {
    db.execSQL("DROP TABLE IF EXISTS asset_market")
    db.execSQL(
        """
            CREATE TABLE asset_market (
                asset_id TEXT NOT NULL,
                marketCap REAL,
                marketCapFdv REAL,
                marketCapRank INTEGER,
                totalVolume REAL,
                circulatingSupply REAL,
                totalSupply REAL,
                maxSupply REAL,
                allTimeHigh REAL,
                allTimeHighDate INTEGER,
                allTimeHighChangePercentage REAL,
                allTimeLow REAL,
                allTimeLowDate INTEGER,
                allTimeLowChangePercentage REAL,
                PRIMARY KEY(asset_id),
                FOREIGN KEY (asset_id) REFERENCES asset(id) ON DELETE CASCADE
            )
        """,
    )
}
