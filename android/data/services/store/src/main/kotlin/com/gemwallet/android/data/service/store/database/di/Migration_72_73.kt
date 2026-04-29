package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_72_73 : Migration(72, 73) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `addresses` (
                    `chain` TEXT NOT NULL,
                    `address` TEXT NOT NULL,
                    `wallet_id` TEXT,
                    `name` TEXT NOT NULL,
                    `type` TEXT,
                    `status` TEXT NOT NULL,
                    PRIMARY KEY(`chain`, `address`),
                    FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`)
                        ON UPDATE CASCADE ON DELETE CASCADE
                )
            """.trimIndent()
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_addresses_wallet_id` ON `addresses` (`wallet_id`)")
        db.execSQL(
            """
                INSERT OR REPLACE INTO `addresses` (`chain`, `address`, `wallet_id`, `name`, `type`, `status`)
                SELECT a.chain, a.address, a.wallet_id, w.name, 'InternalWallet', 'Verified'
                FROM accounts a INNER JOIN wallets w ON a.wallet_id = w.id
            """.trimIndent()
        )
    }
}
