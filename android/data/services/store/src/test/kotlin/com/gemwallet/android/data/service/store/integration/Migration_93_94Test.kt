package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_93_94
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_93_94Test {
    private val testDb = "migration-93-94-test"

    @get:Rule
    val helper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java,
        emptyList(),
        FrameworkSQLiteOpenHelperFactory(),
    )

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun droppingTheWriteOnlyColumnsKeepsEveryRowAndItsForeignKeys() {
        helper.createDatabase(testDb, 93).use { database ->
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, is_earn_enabled, rank, updated_at, associations) " +
                    "VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 0, 0, 0, 0, 0, 7, 1234, '[]')",
            )
            database.execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) VALUES ('tx-1', 'bitcoin', 'ethereum', '1000', '500')")
            database.execSQL(
                "INSERT INTO nft_collections (id, name, description, chain, contractAddress, imageUrl, previewImageUrl, originalSourceUrl, status, links) " +
                    "VALUES ('collection-1', 'Punks', 'desc', 'bitcoin', '0xabc', 'image', 'preview', 'original', 'verified', NULL)",
            )
            database.execSQL(
                "INSERT INTO nft_assets (id, collection_id, token_id, token_type, name, description, chain, contract_address, image_url, preview_image_url, original_image_url, attributes) " +
                    "VALUES ('asset-1', 'collection-1', '1', 'ERC721', 'Punk', 'desc', 'bitcoin', '0xabc', 'image', 'preview', 'original', NULL)",
            )
        }

        helper.runMigrationsAndValidate(testDb, 94, true, Migration_93_94).use { database ->
            assertEquals(
                listOf(listOf("bitcoin", "Bitcoin", "BTC", "8", "7", "[]")),
                database.rows("SELECT id, name, symbol, decimals, rank, associations FROM asset"),
            )
            assertEquals(listOf(listOf("tx-1", "bitcoin", "ethereum")), database.rows("SELECT * FROM tx_swap_metadata"))
            assertEquals(
                listOf(listOf("collection-1", "Punks", "bitcoin", "image", "verified")),
                database.rows("SELECT id, name, chain, imageUrl, status FROM nft_collections"),
            )
            assertEquals(
                listOf(listOf("asset-1", "collection-1", "1", "bitcoin", "image")),
                database.rows("SELECT id, collection_id, token_id, chain, image_url FROM nft_assets"),
            )
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))

            database.execSQL("PRAGMA foreign_keys = ON")
            database.execSQL("DELETE FROM nft_collections WHERE id = 'collection-1'")
            assertEquals(emptyList<List<String?>>(), database.rows("SELECT id FROM nft_assets"))
            database.execSQL("DELETE FROM asset WHERE id = 'bitcoin'")
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))
        }
    }

    private fun SupportSQLiteDatabase.rows(query: String): List<List<String?>> = query(query).use { cursor ->
        buildList {
            while (cursor.moveToNext()) {
                add((0 until cursor.columnCount).map { if (cursor.isNull(it)) null else cursor.getString(it) })
            }
        }
    }
}
