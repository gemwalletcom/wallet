package com.gemwallet.android.service.store

import android.content.Context
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_79_80
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_79_80Test {

    private val testDb = "migration-79-80-test"

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
    fun migrate79To80_dropsAssetsPriorityAndCreatesUnifiedSearchPriority() = runBlocking {
        helper.createDatabase(testDb, 79).apply {
            execSQL("INSERT INTO assets_priority (`query`, asset_id, priority) VALUES ('btc', 'bitcoin', 0)")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 80, true, Migration_79_80)

        assertFalse(migratedDb.hasTable("assets_priority"))
        assertTrue(migratedDb.hasTable("search_priority"))
        assertTrue(migratedDb.hasColumn("search_priority", "query"))
        assertTrue(migratedDb.hasColumn("search_priority", "type"))
        assertTrue(migratedDb.hasColumn("search_priority", "item_id"))
        assertTrue(migratedDb.hasColumn("search_priority", "priority"))

        migratedDb.execSQL("INSERT INTO search_priority (`query`, type, item_id, priority) VALUES ('btc', 'asset', 'bitcoin', 0)")
        migratedDb.execSQL("INSERT INTO search_priority (`query`, type, item_id, priority) VALUES ('btc', 'perpetual', 'hypercore_perpetual::BTC', 0)")
        assertEquals(2, migratedDb.longForQuery("SELECT COUNT(*) FROM search_priority"))
        migratedDb.close()
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
}
