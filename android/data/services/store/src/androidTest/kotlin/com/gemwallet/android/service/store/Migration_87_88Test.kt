package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_87_88
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
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
    fun migrate87To88_keysBannersByCoreIdentifier() {
        helper.createDatabase(testDb, 87).apply {
            execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at, associations) " +
                    "VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 0, 0, 0, 0, 1, 0, '[]')"
            )
            execSQL("INSERT INTO banners (wallet_id, asset_id, chain, state, event) VALUES ('', 'bitcoin', NULL, 'Cancelled', 'Stake')")
            execSQL("INSERT INTO banners (wallet_id, asset_id, chain, state, event) VALUES ('multicoin_wallet-1', '', NULL, 'AlwaysActive', 'AccountBlockedMultiSignature')")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 88, true, Migration_87_88)
        migratedDb.query("SELECT id, wallet_id, asset_id, state FROM banners ORDER BY id").use {
            assertEquals(2, it.count)
            it.moveToFirst()
            assertEquals("bitcoin_stake", it.getString(0))
            assertNull(it.getString(1))
            assertEquals("bitcoin", it.getString(2))
            assertEquals("Cancelled", it.getString(3))
            it.moveToNext()
            assertEquals("multicoin_wallet-1_accountBlockedMultiSignature", it.getString(0))
            assertEquals("multicoin_wallet-1", it.getString(1))
            assertNull(it.getString(2))
        }
        migratedDb.close()
    }
}
