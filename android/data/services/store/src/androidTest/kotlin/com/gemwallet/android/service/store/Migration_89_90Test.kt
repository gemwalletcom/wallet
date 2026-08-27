package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_89_90
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_89_90Test {

    private val testDb = "migration-89-90-test"

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
    fun migrate89To90_addsPendingUnconfirmedAndEarnColumns() {
        helper.createDatabase(testDb, 89).close()

        val migratedDb = helper.runMigrationsAndValidate(testDb, 90, true, Migration_89_90)
        val columns = mutableMapOf<String, String>()
        migratedDb.query("PRAGMA table_info(`balances`)").use {
            while (it.moveToNext()) {
                columns[it.getString(1)] = it.getString(4) ?: ""
            }
        }
        assertEquals("'0'", columns["pending_unconfirmed"])
        assertEquals("0.0", columns["pending_unconfirmed_amount"])
        assertEquals("'0'", columns["earn"])
        assertEquals("0.0", columns["earn_amount"])
        migratedDb.close()
    }
}
