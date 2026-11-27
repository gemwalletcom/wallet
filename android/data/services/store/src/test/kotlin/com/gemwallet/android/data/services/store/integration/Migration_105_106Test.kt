package com.gemwallet.android.data.services.store.integration

import androidx.sqlite.db.SupportSQLiteDatabase
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
    fun negativeFeeBecomesZero() {
        helper.createDatabase(testDb, 105).use { database ->
            database.execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('multicoin', 'Wallet', 'Multicoin', 0, 0, 0, 'Import')")
            database.execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank) " +
                    "VALUES ('sui', 'Asset', 'AST', 9, 'NATIVE', 'sui', 1, 0, 0, 0, 0, 0)",
            )
            database.insertTransaction("negative", "-858240")
            database.insertTransaction("paid", "1750000")
        }

        helper.runMigrationsAndValidate(testDb, 106, true, Migration_105_106).use { database ->
            assertEquals(listOf(listOf("negative", "0"), listOf("paid", "1750000")), database.rows("SELECT hash, fee FROM transactions ORDER BY hash"))
        }
    }

    private fun SupportSQLiteDatabase.insertTransaction(hash: String, fee: String) {
        execSQL(
            "INSERT INTO transactions (id, walletId, hash, assetId, feeAssetId, owner, recipient, state, type, blockNumber, sequence, fee, value, direction, createdAt, updatedAt) " +
                "VALUES (?, 'multicoin', ?, 'sui', 'sui', 'owner', 'recipient', 'confirmed', 'transfer', '1', '1', ?, '1', 'outgoing', 1, 1)",
            arrayOf("sui_$hash", hash, fee),
        )
    }
}
