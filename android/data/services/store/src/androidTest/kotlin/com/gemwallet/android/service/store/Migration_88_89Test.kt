package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_88_89
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_88_89Test {

    private val testDb = "migration-88-89-test"

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
    fun migrate88To89_addsEarnColumns() {
        helper.createDatabase(testDb, 88).apply {
            execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at, associations) " +
                    "VALUES ('ethereum', 'Ethereum', 'ETH', 18, 'NATIVE', 'ethereum', 1, 0, 0, 0, 0, 1, 0, '[]')"
            )
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 89, true, Migration_88_89)
        migratedDb.query("SELECT is_earn_enabled, earn_apr FROM asset WHERE id = 'ethereum'").use {
            assertTrue(it.moveToFirst())
            assertEquals(0, it.getInt(0))
            assertTrue(it.isNull(1))
        }
        migratedDb.close()
    }
}
