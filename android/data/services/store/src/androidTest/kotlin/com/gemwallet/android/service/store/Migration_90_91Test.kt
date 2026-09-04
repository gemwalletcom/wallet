package com.gemwallet.android.service.store

import android.content.Context
import androidx.core.content.edit
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.di.Migration_90_91
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_90_91Test {

    private val testDb = "migration-90-91-test"

    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    private val preferences = context.getSharedPreferences("gemstone_preferences", Context.MODE_PRIVATE)

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
        preferences.edit(commit = true) { clear() }
    }

    @Test
    fun migrate90To91_movesSessionIntoPreferences() {
        helper.createDatabase(testDb, 90).apply {
            execSQL("INSERT INTO session (id, wallet_id, currency) VALUES (1, 'multicoin_0xsecond', 'EUR')")
            close()
        }

        helper.runMigrationsAndValidate(testDb, 91, true, Migration_90_91(context)).close()

        assertEquals("multicoin_0xsecond", preferences.getString("current_wallet_id", null))
        assertEquals("EUR", preferences.getString("currency", null))
    }

    @Test
    fun migrate90To91_keepsAnAlreadyStoredCurrency() {
        preferences.edit(commit = true) { putString("currency", "GBP") }
        helper.createDatabase(testDb, 90).apply {
            execSQL("INSERT INTO session (id, wallet_id, currency) VALUES (1, 'multicoin_0xfirst', 'EUR')")
            close()
        }

        helper.runMigrationsAndValidate(testDb, 91, true, Migration_90_91(context)).close()

        assertEquals("multicoin_0xfirst", preferences.getString("current_wallet_id", null))
        assertEquals("GBP", preferences.getString("currency", null))
    }

    @Test
    fun migrate90To91_writesNothingWithoutASession() {
        helper.createDatabase(testDb, 90).close()

        helper.runMigrationsAndValidate(testDb, 91, true, Migration_90_91(context)).close()

        assertNull(preferences.getString("current_wallet_id", null))
        assertNull(preferences.getString("currency", null))
    }
}
