package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_78_79 : Migration(78, 79) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `contacts` (" +
                "`id` TEXT NOT NULL, " +
                "`name` TEXT NOT NULL, " +
                "`description` TEXT, " +
                "`createdAt` INTEGER NOT NULL, " +
                "`updatedAt` INTEGER NOT NULL, " +
                "PRIMARY KEY(`id`))"
        )
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `contacts_addresses` (" +
                "`id` TEXT NOT NULL, " +
                "`contactId` TEXT NOT NULL, " +
                "`address` TEXT NOT NULL, " +
                "`chain` TEXT NOT NULL, " +
                "`memo` TEXT, " +
                "PRIMARY KEY(`id`), " +
                "FOREIGN KEY(`contactId`) REFERENCES `contacts`(`id`) ON UPDATE CASCADE ON DELETE CASCADE)"
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_contacts_addresses_contactId` ON `contacts_addresses` (`contactId`)")
    }
}
