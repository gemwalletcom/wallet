package com.gemwallet.android.data.services.store.integration

import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.db.SimpleSQLiteQuery
import androidx.sqlite.db.framework.FrameworkSQLiteOpenHelperFactory
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.buildTransactionListSql
import com.gemwallet.android.data.services.store.database.di.Migration_99_100
import com.gemwallet.android.ext.GemConstants
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Migration_99_100Test {
    private val testDb = "migration-99-100-test"

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
    fun theWalletHistoryIsReadNewestFirstFromTheIndex() {
        helper.createDatabase(testDb, 99).close()

        helper.runMigrationsAndValidate(testDb, 100, true, Migration_99_100).use { database ->
            val history = buildTransactionListSql(WalletId("wallet-1"), null, GemConstants.transactionsListLimit)
            database.query(SimpleSQLiteQuery("EXPLAIN QUERY PLAN ${history.sql}", history.args.toTypedArray())).use { cursor ->
                val plan = buildList { while (cursor.moveToNext()) add(cursor.getString(cursor.getColumnIndexOrThrow("detail"))) }
                assertTrue(plan.toString(), plan.any { it.contains("index_transactions_walletId_createdAt") })
                assertTrue(plan.toString(), plan.none { it.contains("TEMP B-TREE FOR ORDER BY") })
            }
        }
    }
}
