package com.gemwallet.android.service.store

import android.content.Context
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_77_78
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_77_78Test {

    private val testDb = "migration-77-78-test"

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
    fun migrate77To78_addsNullableImageUrlColumnAndKeepsExistingWallets() = runBlocking {
        helper.createDatabase(testDb, 77).apply {
            seedWallet("wallet-1")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 78, true, Migration_77_78)

        assertTrue(migratedDb.hasColumn("wallets", "image_url"))
        assertNull(migratedDb.stringForQuery("SELECT image_url FROM wallets WHERE id = 'wallet-1'"))

        migratedDb.execSQL("UPDATE wallets SET image_url = 'avatar.png' WHERE id = 'wallet-1'")
        assertEquals("avatar.png", migratedDb.stringForQuery("SELECT image_url FROM wallets WHERE id = 'wallet-1'"))
        migratedDb.close()
    }

    private fun SupportSQLiteDatabase.seedWallet(id: String) {
        execSQL("INSERT INTO wallets (id, name, type, position, pinned, `index`, source) VALUES ('$id', 'Wallet', 'Multicoin', 0, 0, 0, 'Import')")
    }

    private fun SupportSQLiteDatabase.stringForQuery(query: String): String? {
        val cursor = query(query)
        return cursor.use {
            assertTrue(it.moveToFirst())
            if (it.isNull(0)) null else it.getString(0)
        }
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
}
