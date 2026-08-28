package com.gemwallet.android.service.store

import android.content.Context
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_88_89
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_88_89Test {

    private val testDb = "migration-88-89-test"
    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    @get:Rule
    val helper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java,
        emptyList(),
        FrameworkSQLiteOpenHelperFactory(),
    )

    @Before
    fun setUp() {
        context.deleteDatabase(testDb)
        context.getSharedPreferences("node-config", Context.MODE_PRIVATE).edit().clear().commit()
    }

    @Test
    fun migrate88To89_movesStoredNodesIntoNodeConfig() {
        helper.createDatabase(testDb, 88).apply {
            execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://custom.example', 'Active', 0, 'ethereum')")
            execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://inactive.example', 'Inactive', 2, 'ethereum')")
            execSQL("INSERT INTO nodes (url, status, priority, chain) VALUES ('https://solana.example', 'Active', 0, 'solana')")
            close()
        }

        val migratedDb = helper.runMigrationsAndValidate(testDb, 89, true, Migration_88_89(context))
        migratedDb.query("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'nodes'").use {
            it.moveToFirst()
            assertEquals(0, it.getInt(0))
        }
        migratedDb.close()

        val preferences = context.getSharedPreferences("node-config", Context.MODE_PRIVATE)
        assertEquals(
            listOf(Node("https://custom.example", NodeState.Active, 0), Node("https://inactive.example", NodeState.Inactive, 2)),
            preferences.getString("nodes-ethereum", "")!!.decodeJson<List<Node>>(),
        )
        assertEquals(
            listOf(Node("https://solana.example", NodeState.Active, 0)),
            preferences.getString("nodes-solana", "")!!.decodeJson<List<Node>>(),
        )
        assertFalse(preferences.contains("nodes-bitcoin"))
    }
}
