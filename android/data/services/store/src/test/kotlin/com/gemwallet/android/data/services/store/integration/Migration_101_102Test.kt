package com.gemwallet.android.data.services.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.di.Migration_101_102
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_101_102Test {
    private val testDb = "migration-101-102-test"

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
    fun bannersAreRebuiltEmptyWithTheirWalletForeignKey() {
        helper.createDatabase(testDb, 101).use { database ->
            database.execSQL("INSERT INTO `banners` (`id`, `wallet_id`, `asset_id`, `state`, `event`) VALUES ('orphan', 'deleted-wallet', NULL, 'AlwaysActive', 'AccountBlockedMultiSignature')")
        }

        helper.runMigrationsAndValidate(testDb, 102, true, Migration_101_102).use { database ->
            database.query("SELECT COUNT(*) FROM `banners`").use { cursor ->
                cursor.moveToFirst()
                assertEquals(0, cursor.getInt(0))
            }
        }
    }
}
