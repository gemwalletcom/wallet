package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_83_84
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_83_84Test {

    private val testDb = "migration-83-84-test"

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
    fun migrate83To84_addsEmptyAssetAssociations() {
        helper.createDatabase(testDb, 83).apply {
            execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) " +
                    "VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 0, 0, 0, 0, 1, 0)"
            )
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 84, true, Migration_83_84)
        val cursor = migratedDb.query("SELECT associations FROM asset WHERE id = 'bitcoin'")

        cursor.use {
            it.moveToFirst()
            assertEquals("[]", it.getString(0))
        }
        migratedDb.close()
    }
}
