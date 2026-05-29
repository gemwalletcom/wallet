package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_76_77 : Migration(76, 77) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `room_connection`")
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `wallets_connections` (" +
                "`id` TEXT NOT NULL, " +
                "`wallet_id` TEXT NOT NULL, " +
                "`session_id` TEXT NOT NULL, " +
                "`state` TEXT NOT NULL, " +
                "`chains` TEXT NOT NULL, " +
                "`created_at` INTEGER NOT NULL, " +
                "`expire_at` INTEGER NOT NULL, " +
                "`app_name` TEXT NOT NULL, " +
                "`app_description` TEXT NOT NULL, " +
                "`app_url` TEXT NOT NULL, " +
                "`app_icon` TEXT NOT NULL, " +
                "`redirect_native` TEXT, " +
                "`redirect_universal` TEXT, " +
                "PRIMARY KEY(`id`), " +
                "FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE)"
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_wallets_connections_wallet_id` ON `wallets_connections` (`wallet_id`)")
    }
}
