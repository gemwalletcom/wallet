package com.gemwallet.android.data.service.store.database.di

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

object Migration_69_70 : Migration(69, 70) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("DROP TABLE IF EXISTS `nft_association`")
        db.execSQL("DROP TABLE IF EXISTS `nft_assets_associations`")
        db.execSQL("DROP TABLE IF EXISTS `nft_attributes`")
        db.execSQL("DROP TABLE IF EXISTS `nft_asset`")
        db.execSQL("DROP TABLE IF EXISTS `nft_assets`")
        db.execSQL("DROP TABLE IF EXISTS `nft_collection_link`")
        db.execSQL("DROP TABLE IF EXISTS `nft_collection`")
        db.execSQL("DROP TABLE IF EXISTS `nft_collections`")

        db.execSQL(
            """CREATE TABLE IF NOT EXISTS `nft_collections` (
                `id` TEXT NOT NULL,
                `name` TEXT NOT NULL,
                `description` TEXT,
                `chain` TEXT NOT NULL,
                `contractAddress` TEXT NOT NULL,
                `imageUrl` TEXT NOT NULL,
                `previewImageUrl` TEXT NOT NULL,
                `originalSourceUrl` TEXT NOT NULL,
                `status` TEXT,
                `links` TEXT,
                PRIMARY KEY(`id`),
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )"""
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_collections_chain` ON `nft_collections` (`chain`)")
        db.execSQL(
            """CREATE TABLE IF NOT EXISTS `nft_assets` (
                `id` TEXT NOT NULL,
                `collection_id` TEXT NOT NULL,
                `token_id` TEXT NOT NULL,
                `token_type` TEXT NOT NULL,
                `name` TEXT NOT NULL,
                `description` TEXT,
                `chain` TEXT NOT NULL,
                `contract_address` TEXT,
                `image_url` TEXT NOT NULL,
                `preview_image_url` TEXT NOT NULL,
                `original_image_url` TEXT NOT NULL,
                `attributes` TEXT,
                PRIMARY KEY(`id`),
                FOREIGN KEY(`collection_id`) REFERENCES `nft_collections`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                FOREIGN KEY(`chain`) REFERENCES `asset`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )"""
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_assets_collection_id` ON `nft_assets` (`collection_id`)")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_assets_chain` ON `nft_assets` (`chain`)")
        db.execSQL(
            """CREATE TABLE IF NOT EXISTS `nft_assets_associations` (
                `wallet_id` TEXT NOT NULL,
                `asset_id` TEXT NOT NULL,
                PRIMARY KEY(`wallet_id`, `asset_id`),
                FOREIGN KEY(`asset_id`) REFERENCES `nft_assets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE,
                FOREIGN KEY(`wallet_id`) REFERENCES `wallets`(`id`) ON UPDATE CASCADE ON DELETE CASCADE
            )"""
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_nft_assets_associations_asset_id` ON `nft_assets_associations` (`asset_id`)")
    }
}
