package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_88_89
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
    fun migrate88To89_recreatesBannersWithoutChain() {
        helper.createDatabase(testDb, 88).apply {
            execSQL("INSERT INTO banners (id, wallet_id, asset_id, chain, state, event) VALUES ('stale', 'wallet-1', NULL, 'ethereum', 'AlwaysActive', 'AccountBlockedMultiSignature')")
            close()
        }

        helper.runMigrationsAndValidate(testDb, 89, true, Migration_88_89).close()
    }
}
