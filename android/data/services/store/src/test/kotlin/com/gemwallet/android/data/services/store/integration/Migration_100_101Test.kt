package com.gemwallet.android.data.services.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.di.Migration_100_101
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_100_101Test {
    private val testDb = "migration-100-101-test"

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
    fun aStoredCollectibleGainsAnEmptyResourceUntilItsNextSync() {
        helper.createDatabase(testDb, 100).use { database ->
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank) " +
                    "VALUES ('ethereum', 'Ethereum', 'ETH', 18, 'NATIVE', 'ethereum', 1, 0, 0, 0, 0, 0)",
            )
            database.execSQL("INSERT INTO nft_collections (id, name, chain, contractAddress, imageUrl) VALUES ('collection', 'Collection', 'ethereum', '0xcontract', '')")
            database.execSQL("INSERT INTO nft_assets (id, collection_id, token_id, token_type, name, chain, image_url) VALUES ('asset', 'collection', '1', 'ERC721', 'Asset', 'ethereum', 'https://preview')")
        }

        helper.runMigrationsAndValidate(testDb, 101, true, Migration_100_101).use { database ->
            database.query("SELECT resource_url, resource_mime_type FROM nft_assets WHERE id = 'asset'").use { cursor ->
                cursor.moveToFirst()
                assertEquals("" to "", cursor.getString(0) to cursor.getString(1))
            }
        }
    }
}
