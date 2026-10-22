package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.di.Migration_102_103
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_102_103Test {
    private val testDb = "migration-102-103-test"

    @get:Rule
    val helper = gemDatabaseMigrationTestHelper()

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun aStoredMarketStaysEnabledUntilItsNextSync() {
        helper.createDatabase(testDb, 102).use { database ->
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank) " +
                    "VALUES ('hypercore_perpetual::BTC', 'BTC', 'BTC', 5, 'PERPETUAL', 'hypercore', 0, 0, 0, 0, 0, 0)",
            )
            database.execSQL(
                "INSERT INTO perpetuals (id, name, provider, assetId, identifier, price, pricePercentChange24h, openInterest, volume24h, funding, maxLeverage, isIsolatedOnly, isPinned) " +
                    "VALUES ('hypercore_BTC', 'BTC', 'hypercore', 'hypercore_perpetual::BTC', '0', 1.0, 0.0, 0.0, 0.0, 0.0, 40, 0, 0)",
            )
        }

        helper.runMigrationsAndValidate(testDb, 103, true, Migration_102_103).use { database ->
            database.query("SELECT isEnabled FROM perpetuals WHERE id = 'hypercore_BTC'").use { cursor ->
                cursor.moveToFirst()
                assertEquals(1, cursor.getInt(0))
            }
        }
    }

    @Test
    fun aPriceWriteFindsItsMarketThroughTheNameIndex() {
        helper.createDatabase(testDb, 102).close()

        helper.runMigrationsAndValidate(testDb, 103, true, Migration_102_103).use { database ->
            database.query("EXPLAIN QUERY PLAN UPDATE perpetuals SET price = 1.0 WHERE name = 'BTC'").use { cursor ->
                val plan = buildList { while (cursor.moveToNext()) add(cursor.getString(cursor.getColumnIndexOrThrow("detail"))) }
                assertTrue(plan.toString(), plan.any { it.contains("index_perpetuals_name") })
            }
        }
    }
}
