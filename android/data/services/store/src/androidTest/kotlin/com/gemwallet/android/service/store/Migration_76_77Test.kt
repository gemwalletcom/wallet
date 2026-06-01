package com.gemwallet.android.service.store

import android.content.Context
import androidx.room.Room
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_76_77
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_76_77Test {

    private val testDb = "migration-76-77-test"

    @get:Rule
    val helper: MigrationTestHelper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java,
        emptyList(),
        FrameworkSQLiteOpenHelperFactory()
    )

    private lateinit var context: Context

    @Before
    fun setUp() {
        context = ApplicationProvider.getApplicationContext()
        context.deleteDatabase(testDb)
    }

    @Test
    fun migrate76To77_recreatesConnectionsTableWithChainsAndWalletForeignKey() = runBlocking {
        helper.createDatabase(testDb, 76).apply {
            seedWallet("wallet-1")
            insertLegacyConnection("topic-legacy")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 77, true, Migration_76_77)
        assertFalse(migratedDb.hasTable("room_connection"))
        assertTrue(migratedDb.hasColumn("wallets_connections", "chains"))
        assertEquals(0, migratedDb.longForQuery("SELECT COUNT(*) FROM wallets_connections"))
        assertTrue(migratedDb.hasForeignKey("wallets_connections", "wallet_id", "wallets", "id"))
        assertTrue(migratedDb.hasIndex("wallets_connections", "index_wallets_connections_wallet_id"))
        migratedDb.insertConnectionWithChains("topic-chains", "[\"ethereum\",\"solana\"]")
        migratedDb.close()

        val roomDb = Room.databaseBuilder(context, GemDatabase::class.java, testDb)
            .addMigrations(Migration_76_77)
            .allowMainThreadQueries()
            .build()
        val withChains = roomDb.connectionsDao().getBySessionId("topic-chains")
        roomDb.close()

        assertEquals(listOf(Chain.Ethereum, Chain.Solana), withChains?.chains)
    }

    private fun SupportSQLiteDatabase.seedWallet(id: String) {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('$id', 'Wallet', 'Multicoin', 0, 0, 0, 'Import')")
    }

    private fun SupportSQLiteDatabase.insertLegacyConnection(topic: String) {
        execSQL(
            "INSERT INTO room_connection (id, wallet_id, session_id, state, created_at, expire_at, app_name, app_description, app_url, app_icon, redirect_native, redirect_universal) " +
                "VALUES ('$topic', 'wallet-1', '$topic', 'Active', 1000, 2000, 'App', 'Desc', 'https://app.example', 'https://app.example/icon.png', NULL, NULL)"
        )
    }

    private fun SupportSQLiteDatabase.insertConnectionWithChains(topic: String, chains: String) {
        execSQL(
            "INSERT INTO wallets_connections (id, wallet_id, session_id, state, chains, created_at, expire_at, app_name, app_description, app_url, app_icon, redirect_native, redirect_universal) " +
                "VALUES ('$topic', 'wallet-1', '$topic', 'Active', '$chains', 1000, 2000, 'App', 'Desc', 'https://app.example', 'https://app.example/icon.png', NULL, NULL)"
        )
    }

    private fun SupportSQLiteDatabase.longForQuery(query: String): Long {
        val cursor = query(query)
        return cursor.use {
            assertTrue(it.moveToFirst())
            it.getLong(0)
        }
    }

    private fun SupportSQLiteDatabase.hasTable(name: String): Boolean {
        val cursor = query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = '$name'")
        return cursor.use { it.moveToFirst() }
    }

    private fun SupportSQLiteDatabase.hasColumn(table: String, column: String): Boolean {
        val cursor = query("PRAGMA table_info($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(1) == column) {
                    return@use true
                }
            }
            false
        }
    }

    private fun SupportSQLiteDatabase.hasIndex(table: String, name: String): Boolean {
        val cursor = query("PRAGMA index_list($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(1) == name) {
                    return@use true
                }
            }
            false
        }
    }

    private fun SupportSQLiteDatabase.hasForeignKey(table: String, from: String, toTable: String, to: String): Boolean {
        val cursor = query("PRAGMA foreign_key_list($table)")
        return cursor.use {
            while (it.moveToNext()) {
                if (it.getString(2) == toTable && it.getString(3) == from && it.getString(4) == to) {
                    return@use true
                }
            }
            false
        }
    }
}
