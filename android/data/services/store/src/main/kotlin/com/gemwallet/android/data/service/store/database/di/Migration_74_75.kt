package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_74_75 : Migration(74, 75) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `stake_delegation_base`")
        db.execSQL("DROP TABLE IF EXISTS `stake_delegation_validator`")

        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `stake_validators` (
                    `id` TEXT NOT NULL,
                    `assetId` TEXT NOT NULL,
                    `validatorId` TEXT NOT NULL,
                    `name` TEXT NOT NULL,
                    `isActive` INTEGER NOT NULL,
                    `commission` REAL NOT NULL,
                    `apr` REAL NOT NULL,
                    `providerType` TEXT NOT NULL,
                    PRIMARY KEY(`id`),
                    FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_stake_validators_assetId` ON `stake_validators` (`assetId`)")

        db.execSQL(
            """
                CREATE TABLE IF NOT EXISTS `stake_delegations` (
                    `id` TEXT NOT NULL,
                    `walletId` TEXT NOT NULL,
                    `assetId` TEXT NOT NULL,
                    `validatorId` TEXT NOT NULL,
                    `state` TEXT NOT NULL,
                    `delegationId` TEXT NOT NULL,
                    `balance` TEXT NOT NULL,
                    `shares` TEXT NOT NULL,
                    `rewards` TEXT NOT NULL,
                    `completionDate` INTEGER,
                    PRIMARY KEY(`walletId`, `id`),
                    FOREIGN KEY(`walletId`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(`assetId`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                    FOREIGN KEY(`validatorId`) REFERENCES `stake_validators`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
                )
            """
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_stake_delegations_assetId` ON `stake_delegations` (`assetId`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_stake_delegations_validatorId` ON `stake_delegations` (`validatorId`)")
    }
}
