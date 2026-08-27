package com.gemwallet.android.service.store

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_91_92
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_91_92Test {

    private val testDb = "migration-91-92-test"

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
    fun migrate91To92_keepsSessionCurrencyWithoutWalletAndNodesPerChain() {
        helper.createDatabase(testDb, 91).use { db ->
            db.execSQL("INSERT INTO `session` (`id`, `wallet_id`, `currency`) VALUES (1, 'wallet-1', 'USD')")
            db.execSQL("INSERT INTO `asset` (`id`, `chain`, `name`, `symbol`, `decimals`, `type`) VALUES ('ethereum', 'ethereum', 'Ethereum', 'ETH', 18, 'NATIVE')")
            db.execSQL("INSERT INTO `nodes` (`url`, `status`, `priority`, `chain`) VALUES ('https://rpc.example', 'active', 1, 'ethereum')")
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 92, true, Migration_91_92)
        migratedDb.execSQL("UPDATE `session` SET `wallet_id` = NULL WHERE `id` = 1")
        migratedDb.query("SELECT `wallet_id`, `currency` FROM `session`").use {
            it.moveToFirst()
            assertEquals(true, it.isNull(0))
            assertEquals("USD", it.getString(1))
        }
        migratedDb.query("SELECT COUNT(*) FROM `nodes` WHERE `chain` = 'ethereum' AND `url` = 'https://rpc.example'").use {
            it.moveToFirst()
            assertEquals(1, it.getInt(0))
        }
        migratedDb.close()
    }
}
