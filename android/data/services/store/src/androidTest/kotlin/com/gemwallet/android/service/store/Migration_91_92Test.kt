package com.gemwallet.android.service.store

import android.database.sqlite.SQLiteConstraintException
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_91_92
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
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
    fun migrationPreservesTransactionsAndAssignsUniqueStableIds() {
        val columns = "id, walletId, hash, assetId, feeAssetId, owner, recipient, contract, metadata, state, type, blockNumber, sequence, fee, value, payload, direction, createdAt, updatedAt, estimatedConfirmationInSeconds"
        val original = helper.createDatabase(testDb, 91).use { database ->
            database.execSQL("INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank, updated_at) VALUES ('bitcoin', 'Bitcoin', 'BTC', 8, 'NATIVE', 'bitcoin', 1, 0, 0, 0, 0, 0, 0)")
            listOf("wallet-1", "wallet-2").forEach { walletId ->
                database.execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES (?, 'Wallet', 'multicoin', 0, 0, 0, 'Import')", arrayOf(walletId))
                database.execSQL("INSERT INTO transactions ($columns) VALUES ('bitcoin_shared', ?, 'shared', 'bitcoin', 'bitcoin', 'sender', 'recipient', 'contract', 'metadata', 'pending', 'swap', '123', '7', '25', '1000', 'memo', 'outgoing', 1234, 5678, 30)", arrayOf(walletId))
            }
            database.execSQL("INSERT INTO addresses (chain, address, walletId, name, type, status) VALUES ('bitcoin', 'sender', 'wallet-1', 'Sender', 'InternalWallet', 'Verified'), ('bitcoin', 'recipient', NULL, 'Recipient', 'Contact', 'Verified')")
            database.execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id, from_amount, to_amount) VALUES ('bitcoin_shared', 'bitcoin', 'ethereum', '1000', '500')")
            database.rows("SELECT $columns FROM transactions ORDER BY walletId")
        }

        helper.runMigrationsAndValidate(testDb, 92, true, Migration_91_92).use { database ->
            assertEquals(original, database.rows("SELECT $columns FROM transactions ORDER BY walletId"))
            assertEquals(
                listOf(listOf("recipient", null, "Recipient", "Contact", "Verified"), listOf("sender", "wallet-1", "Sender", "InternalWallet", "Verified")),
                database.rows("SELECT address, walletId, name, type, status FROM addresses ORDER BY address"),
            )
            assertEquals(
                listOf(listOf("wallet-1", "Sender", "Recipient"), listOf("wallet-2", "Sender", "Recipient")),
                database.rows("SELECT tx.walletId, sender.name, recipient.name FROM transactions AS tx JOIN asset ON tx.assetId = asset.id LEFT JOIN addresses AS sender ON sender.chain = asset.chain AND sender.address = tx.owner LEFT JOIN addresses AS recipient ON recipient.chain = asset.chain AND recipient.address = tx.recipient ORDER BY tx.walletId"),
            )
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))
            val ids = database.rows("SELECT recordId FROM transactions").map { it.single()!!.toLong() }
            assertEquals(2, ids.toSet().size)
            assertTrue(ids.all { it > 0 })
            assertEquals(listOf(listOf("bitcoin_shared", "bitcoin", "ethereum", "1000", "500")), database.rows("SELECT * FROM tx_swap_metadata"))
            assertThrows(SQLiteConstraintException::class.java) {
                database.execSQL("INSERT INTO transactions ($columns) SELECT $columns FROM transactions WHERE walletId = 'wallet-1'")
            }
            database.execSQL("PRAGMA foreign_keys = ON")
            database.execSQL("DELETE FROM wallets WHERE id = 'wallet-1'")
            assertEquals(listOf(listOf("wallet-2")), database.rows("SELECT walletId FROM transactions"))
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))
        }
    }

    private fun SupportSQLiteDatabase.rows(query: String): List<List<String?>> = query(query).use { cursor ->
        buildList {
            while (cursor.moveToNext()) {
                add((0 until cursor.columnCount).map { if (cursor.isNull(it)) null else cursor.getString(it) })
            }
        }
    }
}
