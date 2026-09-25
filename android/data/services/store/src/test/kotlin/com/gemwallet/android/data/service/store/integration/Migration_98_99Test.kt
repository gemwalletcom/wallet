package com.gemwallet.android.data.service.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_98_99
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_98_99Test {
    private val testDb = "migration-98-99-test"

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
    fun swapPairsAndMainAssetsMoveIntoTheTransactionAssetsTable() {
        helper.createDatabase(testDb, 98).use { database ->
            database.execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`) VALUES ('wallet-1', 'Main', 'multicoin', 0, 0, 0), ('wallet-2', 'Other', 'multicoin', 1, 0, 1)")
            database.insertTransaction("bitcoin_swap", "wallet-1", "bitcoin", "Swap")
            database.insertTransaction("bitcoin_swap", "wallet-2", "bitcoin", "Swap")
            database.insertTransaction("ethereum_transfer", "wallet-1", "ethereum", "Transfer")
            database.execSQL("INSERT INTO tx_swap_metadata (tx_id, from_asset_id, to_asset_id) VALUES ('bitcoin_swap', 'bitcoin', 'ethereum_0xdac17f958d2ee523a2206206994597c13d831ec7')")
        }

        helper.runMigrationsAndValidate(testDb, 99, true, Migration_98_99).use { database ->
            assertEquals(
                listOf(
                    listOf("bitcoin_swap", "bitcoin"),
                    listOf("bitcoin_swap", "ethereum_0xdac17f958d2ee523a2206206994597c13d831ec7"),
                    listOf("ethereum_transfer", "ethereum"),
                ),
                database.rows("SELECT tx_id, asset_id FROM transactions_assets ORDER BY tx_id, asset_id"),
            )
            assertEquals(emptyList<List<String?>>(), database.rows("SELECT name FROM sqlite_master WHERE name = 'tx_swap_metadata'"))
        }
    }

    private fun SupportSQLiteDatabase.insertTransaction(id: String, walletId: String, assetId: String, type: String) = execSQL(
        "INSERT INTO transactions (id, walletId, hash, assetId, feeAssetId, owner, recipient, state, type, blockNumber, sequence, fee, value, direction, createdAt, updatedAt) " +
            "VALUES ('$id', '$walletId', 'hash', '$assetId', '$assetId', 'owner', 'recipient', 'Confirmed', '$type', '1', '1', '1', '1', 'outgoing', 0, 0)",
    )

    private fun SupportSQLiteDatabase.rows(query: String): List<List<String?>> = query(query).use { cursor ->
        buildList {
            while (cursor.moveToNext()) {
                add((0 until cursor.columnCount).map { cursor.getString(it) })
            }
        }
    }
}
