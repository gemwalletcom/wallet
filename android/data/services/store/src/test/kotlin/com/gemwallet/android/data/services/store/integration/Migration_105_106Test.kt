package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.di.Migration_105_106
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_105_106Test {
    private val testDb = "migration-105-106-test"

    @get:Rule
    val helper = gemDatabaseMigrationTestHelper()

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun lightningRowsAreStoredPerWalletAndDeletedWithIt() {
        helper.createDatabase(testDb, 105).use { database ->
            listOf("multicoin", "other").forEach { id ->
                database.execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES (?, 'Wallet', 'Multicoin', 0, 0, 0, 'Import')", arrayOf(id))
            }
        }

        helper.runMigrationsAndValidate(testDb, 106, true, Migration_105_106).use { database ->
            database.execSQL("PRAGMA foreign_keys = ON")
            listOf("multicoin", "other").forEach { walletId ->
                database.execSQL("INSERT INTO lightning_leaves (wallet_id, id, data) VALUES (?, 'leaf', '{}')", arrayOf(walletId))
                database.execSQL(
                    "INSERT INTO lightning_pending_sends (wallet_id, transfer_id, invoice, amount_sats, expires_at) VALUES (?, 'transfer', 'lnbc1', NULL, 1800000000)",
                    arrayOf(walletId),
                )
            }

            database.execSQL("DELETE FROM wallets WHERE id = 'multicoin'")

            assertEquals(listOf(listOf("other")), database.rows("SELECT wallet_id FROM lightning_leaves"))
            assertEquals(listOf(listOf("other")), database.rows("SELECT wallet_id FROM lightning_pending_sends"))
        }
    }
}
