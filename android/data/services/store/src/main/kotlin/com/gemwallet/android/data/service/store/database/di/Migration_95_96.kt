package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_95_96 : Migration(95, 96) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `balances` ADD COLUMN `metadata` TEXT DEFAULT NULL")
        db.execSQL(
            """
            UPDATE balances SET metadata = json_object(
                'votes', votes,
                'energyAvailable', energy_available,
                'energyTotal', energy_total,
                'bandwidthAvailable', bandwidth_available,
                'bandwidthTotal', bandwidth_total
            )
            WHERE votes != 0 OR energy_available != 0 OR energy_total != 0 OR bandwidth_available != 0 OR bandwidth_total != 0
            """,
        )
        for (column in listOf("votes", "energy_available", "energy_total", "bandwidth_available", "bandwidth_total", "list_position")) {
            db.execSQL("ALTER TABLE `balances` DROP COLUMN `$column`")
        }
    }
}
