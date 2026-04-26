package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.gemwallet.android.data.service.store.database.entities.EXTENDED_TXS_VIEW_NAME
import com.gemwallet.android.data.service.store.database.entities.EXTENDED_TXS_VIEW_SQL

object Migration_70_71 : Migration(70, 71) {
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

        db.execSQL("DROP VIEW IF EXISTS `$EXTENDED_TXS_VIEW_NAME`")
        db.execSQL("CREATE VIEW `$EXTENDED_TXS_VIEW_NAME` AS ${EXTENDED_TXS_VIEW_SQL.trim()}")
    }
}
