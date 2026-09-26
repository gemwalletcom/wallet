package com.gemwallet.android.data.services.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.di.Migration_95_96
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_95_96Test {
    private val testDb = "migration-95-96-test"

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
    fun theBalancesAreKeptAndTheOldColumnsDropped() {
        val kept = "SELECT asset_id, wallet_id, available, available_amount, withdrawableAmount, total_amount, is_active, is_pinned, is_visible, updated_at FROM balances ORDER BY asset_id"
        val before = helper.createDatabase(testDb, 95).use { database ->
            database.seedWallet()
            database.execSQL(balanceRow("tron", "11, 22, 33, 44, 55, 3"))
            database.execSQL(balanceRow("bitcoin", "0, 0, 0, 0, 0, 0"))
            database.execSQL("UPDATE balances SET available = '12345', available_amount = 1.5, withdrawableAmount = 0.25, total_amount = 2.5, is_pinned = 1 WHERE asset_id = 'tron'")
            database.rows(kept)
        }

        helper.runMigrationsAndValidate(testDb, 96, true, Migration_95_96).use { database ->
            assertEquals(before, database.rows(kept))
            assertNull(database.metadata("tron"))

            val columns = database.rows("PRAGMA table_info(balances)").mapNotNull { it.getOrNull(1) }
            for (dropped in listOf("votes", "energy_available", "energy_total", "bandwidth_available", "bandwidth_total", "list_position")) {
                assertEquals("$dropped is gone", false, columns.contains(dropped))
            }
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))
        }
    }

    private fun balanceRow(assetId: String, metadata: String): String {
        val amounts = listOf(
            "available", "frozen", "locked", "staked", "pending",
            "rewards", "reserved", "withdrawable", "pending_unconfirmed", "earn",
        )
        val columns = amounts.flatMap { listOf(it, if (it == "withdrawable") "withdrawableAmount" else "${it}_amount") }
        val metadataColumns = listOf("votes", "energy_available", "energy_total", "bandwidth_available", "bandwidth_total", "list_position")
        val names = (listOf("asset_id", "wallet_id") + columns + listOf("total_amount", "is_active", "is_pinned", "is_visible") + metadataColumns + "updated_at").joinToString()
        val zeroed = columns.joinToString { if (it.endsWith("_amount") || it == "withdrawableAmount") "0" else "'0'" }
        return "INSERT INTO balances ($names) VALUES ('$assetId', 'wallet-1', $zeroed, 0, 1, 0, 1, $metadata, 1)"
    }

    private fun SupportSQLiteDatabase.seedWallet() {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('wallet-1', 'Wallet', 'Multicoin', 0, 0, 0, 'Import')")
        for (id in listOf("tron", "bitcoin")) {
            execSQL(
                "INSERT INTO asset (id, name, symbol, decimals, type, chain, rank, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, is_earn_enabled, has_image) " +
                    "VALUES ('$id', '$id', '$id', 8, 'NATIVE', '$id', 1, 1, 0, 0, 0, 0, 0, 0)",
            )
        }
    }

    private fun SupportSQLiteDatabase.metadata(assetId: String): String? = rows("SELECT metadata FROM balances WHERE asset_id = '$assetId'").single().first()

    private fun SupportSQLiteDatabase.rows(query: String): List<List<String?>> = query(query).use { cursor ->
        buildList {
            while (cursor.moveToNext()) {
                add((0 until cursor.columnCount).map { if (cursor.isNull(it)) null else cursor.getString(it) })
            }
        }
    }
}
