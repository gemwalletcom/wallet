package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_90_91
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_90_91Test {

    private val testDb = "migration-90-91-test"

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
    fun migrate90To91_recreatesBannersWithWalletCascade() {
        helper.createDatabase(testDb, 90).use { db ->
            db.execSQL("INSERT INTO `banners` (`id`, `wallet_id`, `state`, `event`) VALUES ('stale', 'gone', 'active', 'stake')")
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 91, true, Migration_90_91)
        migratedDb.query("SELECT COUNT(*) FROM `banners`").use {
            it.moveToFirst()
            assertEquals(0, it.getInt(0))
        }
        migratedDb.close()
    }
}
