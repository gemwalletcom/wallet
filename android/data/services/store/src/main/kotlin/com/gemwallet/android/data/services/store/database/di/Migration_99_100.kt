package com.gemwallet.android.data.services.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_99_100 : Migration(99, 100) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_transactions_walletId_createdAt` ON `transactions` (`walletId`, `createdAt`)")
        db.execSQL("DROP INDEX IF EXISTS `index_transactions_walletId`")
    }
}
