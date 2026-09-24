package com.gemwallet.android.data.service.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_96_97
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_96_97Test {
    private val testDb = "migration-96-97-test"

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
    fun aStoredMarketKeepsItsValuesAndGainsEmptyUsdColumns() {
        helper.createDatabase(testDb, 96).use { database ->
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, rank, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, is_earn_enabled, has_image) " +
                    "VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 1, 0, 0, 0, 0, 0, 0)",
            )
            database.execSQL("INSERT INTO asset_market (asset_id, marketCap) VALUES ('bitcoin', 900.0)")
        }

        helper.runMigrationsAndValidate(testDb, 97, true, Migration_96_97).use { database ->
            assertEquals(listOf(listOf("1", "1")), database.rows("SELECT marketCap = 900.0, marketCapUsd IS NULL FROM asset_market"))
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
