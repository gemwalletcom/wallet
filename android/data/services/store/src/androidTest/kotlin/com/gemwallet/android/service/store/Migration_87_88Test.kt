package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_87_88
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_87_88Test {

    private val testDb = "migration-87-88-test"

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
    fun migrate87To88_recreatesBannersKeyedByIdentifier() {
        helper.createDatabase(testDb, 87).apply {
            execSQL("INSERT INTO banners (wallet_id, asset_id, chain, state, event) VALUES ('', 'bitcoin', NULL, 'Cancelled', 'Stake')")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 88, true, Migration_87_88)
        migratedDb.query("SELECT COUNT(*) FROM banners").use {
            it.moveToFirst()
            assertEquals(0, it.getInt(0))
        }
        migratedDb.close()
    }
}
