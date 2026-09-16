package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_93_94 : Migration(93, 94) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `asset_new` (`id` TEXT NOT NULL, `name` TEXT NOT NULL, `symbol` TEXT NOT NULL, `decimals` INTEGER NOT NULL,
            `type` TEXT NOT NULL, `chain` TEXT NOT NULL, `is_enabled` INTEGER NOT NULL, `is_buy_enabled` INTEGER NOT NULL,
            `is_sell_enabled` INTEGER NOT NULL, `is_swap_enabled` INTEGER NOT NULL, `is_stake_enabled` INTEGER NOT NULL, `staking_apr` REAL,
            `is_earn_enabled` INTEGER NOT NULL DEFAULT 0, `earn_apr` REAL, `rank` INTEGER NOT NULL,
            `associations` TEXT NOT NULL DEFAULT '[]', PRIMARY KEY(`id`))
            """.trimIndent(),
        )
        db.execSQL(
            """
            INSERT INTO `asset_new` (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled,
            is_stake_enabled, staking_apr, is_earn_enabled, earn_apr, rank, associations)
            SELECT id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled,
            is_stake_enabled, staking_apr, is_earn_enabled, earn_apr, rank, associations FROM `asset`
            """.trimIndent(),
        )
        db.execSQL("DROP TABLE `asset`")
        db.execSQL("ALTER TABLE `asset_new` RENAME TO `asset`")

        db.execSQL("CREATE TABLE IF NOT EXISTS `tx_swap_metadata_new` (`tx_id` TEXT NOT NULL, `from_asset_id` TEXT NOT NULL, `to_asset_id` TEXT NOT NULL, PRIMARY KEY(`tx_id`))")
        db.execSQL("INSERT INTO `tx_swap_metadata_new` (tx_id, from_asset_id, to_asset_id) SELECT tx_id, from_asset_id, to_asset_id FROM `tx_swap_metadata`")
        db.execSQL("DROP TABLE `tx_swap_metadata`")
        db.execSQL("ALTER TABLE `tx_swap_metadata_new` RENAME TO `tx_swap_metadata`")

        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `nft_collections_new` (`id` TEXT NOT NULL, `name` TEXT NOT NULL, `description` TEXT, `chain` TEXT NOT NULL,
            `contractAddress` TEXT NOT NULL, `imageUrl` TEXT NOT NULL, `status` TEXT, `links` TEXT, PRIMARY KEY(`id`),
            FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE )
            """.trimIndent(),
        )
        db.execSQL(
            """
            INSERT INTO `nft_collections_new` (id, name, description, chain, contractAddress, imageUrl, status, links)
            SELECT id, name, description, chain, contractAddress, imageUrl, status, links FROM `nft_collections`
            """.trimIndent(),
        )
        db.execSQL("DROP TABLE `nft_collections`")
        db.execSQL("ALTER TABLE `nft_collections_new` RENAME TO `nft_collections`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_collections_chain` ON `nft_collections` (`chain`)")

        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS `nft_assets_new` (`id` TEXT NOT NULL, `collection_id` TEXT NOT NULL, `token_id` TEXT NOT NULL,
            `token_type` TEXT NOT NULL, `name` TEXT NOT NULL, `description` TEXT, `chain` TEXT NOT NULL, `contract_address` TEXT,
            `image_url` TEXT NOT NULL, `attributes` TEXT, PRIMARY KEY(`id`),
            FOREIGN KEY(`collection_id`) REFERENCES `nft_collections`(`id`) ON UPDATE CASCADE ON DELETE CASCADE ,
            FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE )
            """.trimIndent(),
        )
        db.execSQL(
            """
            INSERT INTO `nft_assets_new` (id, collection_id, token_id, token_type, name, description, chain, contract_address, image_url, attributes)
            SELECT id, collection_id, token_id, token_type, name, description, chain, contract_address, image_url, attributes FROM `nft_assets`
            """.trimIndent(),
        )
        db.execSQL("DROP TABLE `nft_assets`")
        db.execSQL("ALTER TABLE `nft_assets_new` RENAME TO `nft_assets`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_assets_collection_id` ON `nft_assets` (`collection_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_assets_chain` ON `nft_assets` (`chain`)")
    }
}
