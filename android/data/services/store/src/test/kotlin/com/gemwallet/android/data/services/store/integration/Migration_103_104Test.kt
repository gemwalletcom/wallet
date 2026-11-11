package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.di.Migration_103_104
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_103_104Test {
    private val testDb = "migration-103-104-test"

    @get:Rule
    val helper = gemDatabaseMigrationTestHelper()

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun aStoredAssetHasNoCandlesUntilItsNextSync() {
        helper.createDatabase(testDb, 103).use { database ->
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank) " +
                    "VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 0, 0, 0, 0, 0)",
            )
        }

        helper.runMigrationsAndValidate(testDb, 104, true, Migration_103_104).use { database ->
            database.query("SELECT has_candles FROM asset WHERE id = 'bitcoin'").use { cursor ->
                cursor.moveToFirst()
                assertEquals(0, cursor.getInt(0))
            }
        }
    }
}
