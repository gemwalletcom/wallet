package com.gemwallet.android.data.services.store.integration

import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.di.Migration_104_105
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_104_105Test {
    private val testDb = "migration-104-105-test"

    @get:Rule
    val helper = gemDatabaseMigrationTestHelper()

    @Before
    fun setUp() {
        InstrumentationRegistry.getInstrumentation().targetContext.deleteDatabase(testDb)
    }

    @Test
    fun removedChainsLeaveOnlyKeyWalletAccounts() {
        helper.createDatabase(testDb, 104).use { database ->
            database.insertWallet("multicoin", "Multicoin")
            database.insertWallet("view_blast_0xview", "View")
            database.insertWallet("privateKey_blast_0xkey", "PrivateKey")
            listOf(
                "ethereum" to "ethereum",
                "blast" to "blast",
                "blast_0xtoken" to "blast",
                "sei" to "sei",
                "seievm" to "seievm",
                "seievm_0xtoken" to "seievm",
            ).forEach { (assetId, chain) -> database.insertAsset(assetId, chain) }
            listOf(
                "multicoin" to "ethereum",
                "multicoin" to "blast",
                "multicoin" to "sei",
                "multicoin" to "seievm",
                "view_blast_0xview" to "blast",
                "privateKey_blast_0xkey" to "blast",
            ).forEach { (walletId, chain) -> database.insertAccount(walletId, chain) }
            listOf("blast", "blast_0xtoken", "sei", "seievm", "seievm_0xtoken").forEach { database.insertPrice(it) }
            listOf("blast_0xtoken", "sei", "seievm_0xtoken").forEach { database.insertTransaction(it) }
            listOf("blast_0xtoken", "seievm_0xtoken").forEach { database.insertBalance(it) }
        }

        helper.runMigrationsAndValidate(testDb, 105, true, Migration_104_105).use { database ->
            assertEquals(listOf(listOf("multicoin"), listOf("privateKey_blast_0xkey")), database.rows("SELECT id FROM wallets ORDER BY id"))
            assertEquals(
                listOf(listOf("multicoin", "ethereum"), listOf("multicoin", "seievm"), listOf("privateKey_blast_0xkey", "blast")),
                database.rows("SELECT wallet_id, chain FROM accounts ORDER BY wallet_id, chain"),
            )
            assertEquals(
                listOf(listOf("blast"), listOf("ethereum"), listOf("seievm"), listOf("seievm_0xtoken")),
                database.rows("SELECT id FROM asset ORDER BY id"),
            )
            assertEquals(listOf(listOf("seievm"), listOf("seievm_0xtoken")), database.rows("SELECT asset_id FROM prices ORDER BY asset_id"))
            assertEquals(listOf(listOf("seievm_0xtoken")), database.rows("SELECT assetId FROM transactions"))
            assertEquals(listOf(listOf("seievm_0xtoken")), database.rows("SELECT asset_id FROM balances"))
            assertEquals(emptyList<List<String?>>(), database.rows("PRAGMA foreign_key_check"))
        }
    }

    private fun SupportSQLiteDatabase.insertWallet(id: String, type: String) {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES (?, 'Wallet', ?, 0, 0, 0, 'Import')", arrayOf(id, type))
    }

    private fun SupportSQLiteDatabase.insertAsset(id: String, chain: String) {
        execSQL(
            "INSERT INTO asset (id, name, symbol, decimals, type, chain, is_enabled, is_buy_enabled, is_sell_enabled, is_swap_enabled, is_stake_enabled, rank) " +
                "VALUES (?, 'Asset', 'AST', 18, 'NATIVE', ?, 1, 0, 0, 0, 0, 0)",
            arrayOf(id, chain),
        )
    }

    private fun SupportSQLiteDatabase.insertAccount(walletId: String, chain: String) {
        execSQL("INSERT INTO accounts (wallet_id, derivation_path, address, chain) VALUES (?, 'm/44', 'address', ?)", arrayOf(walletId, chain))
    }

    private fun SupportSQLiteDatabase.insertPrice(assetId: String) {
        execSQL("INSERT INTO prices (asset_id, value, usd_value, day_changed, currency) VALUES (?, 1.0, 1.0, 0.0, 'USD')", arrayOf(assetId))
    }

    private fun SupportSQLiteDatabase.insertBalance(assetId: String) {
        execSQL(
            "INSERT INTO balances (asset_id, wallet_id, available, available_amount, frozen, frozen_amount, locked, locked_amount, staked, staked_amount, pending, pending_amount, " +
                "rewards, rewards_amount, reserved, reserved_amount, withdrawable, withdrawableAmount, total_amount, is_active, is_pinned, is_visible) " +
                "VALUES (?, 'multicoin', '1', 1.0, '0', 0.0, '0', 0.0, '0', 0.0, '0', 0.0, '0', 0.0, '0', 0.0, '0', 0.0, 1.0, 1, 0, 1)",
            arrayOf(assetId),
        )
    }

    private fun SupportSQLiteDatabase.insertTransaction(assetId: String) {
        execSQL(
            "INSERT INTO transactions (id, walletId, hash, assetId, feeAssetId, owner, recipient, state, type, blockNumber, sequence, fee, value, direction, createdAt, updatedAt) " +
                "VALUES (?, 'multicoin', ?, ?, ?, 'owner', 'recipient', 'confirmed', 'transfer', '1', '1', '1', '1', 'outgoing', 1, 1)",
            arrayOf("${assetId}_hash", "hash_$assetId", assetId, assetId),
        )
    }
}
