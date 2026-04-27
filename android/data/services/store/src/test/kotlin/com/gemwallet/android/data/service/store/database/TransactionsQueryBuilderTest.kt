package com.gemwallet.android.data.service.store.database

import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionsQueryBuilderTest {

    @Test
    fun emptyFilters_baseQueryWithLimitOnly() {
        val q = buildExtendedTransactionsSql(filters = emptyList())
        assertEquals(
            "SELECT * FROM extended_txs ORDER BY createdAt DESC LIMIT ?",
            q.sql,
        )
        assertEquals(listOf<Any>(DEFAULT_TRANSACTIONS_LIMIT), q.args)
    }

    @Test
    fun emptyChainsOrTypes_areNoOps() {
        val chainsOnly = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Chains(emptyList())),
        )
        val typesOnly = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Types(emptyList())),
        )
        assertFalse(chainsOnly.sql.contains("WHERE"))
        assertFalse(typesOnly.sql.contains("WHERE"))
    }

    @Test
    fun chainsFilter_buildsSubqueryWithEnumName() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Ethereum, Chain.Bitcoin))),
        )
        assertTrue(
            q.sql.contains("assetId IN (SELECT id FROM asset WHERE chain IN (?,?))"),
        )
        assertEquals("Ethereum", q.args[0])
        assertEquals("Bitcoin", q.args[1])
    }

    @Test
    fun typesFilter_buildsInClauseWithEnumNames() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Types(listOf(TransactionType.Transfer, TransactionType.Swap))),
        )
        assertTrue(q.sql.contains("type IN (?,?)"))
        assertEquals("Transfer", q.args[0])
        assertEquals("Swap", q.args[1])
    }

    @Test
    fun assetRankGreaterThan_buildsRankSubquery() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.AssetRankGreaterThan(15)),
        )
        assertTrue(q.sql.contains("assetId IN (SELECT id FROM asset WHERE rank > ?)"))
        assertEquals(15, q.args[0])
    }

    @Test
    fun assetFilter_matchesMainAndSwapAssets_bindsIdThreeTimes() {
        val assetId = AssetId(chain = Chain.Ethereum, tokenId = "0xABC")
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Asset(assetId)),
        )
        assertTrue(
            q.sql.contains("(assetId = ? OR assetIdFrom = ? OR assetIdTo = ?)"),
        )
        assertEquals("ethereum_0xABC", q.args[0])
        assertEquals("ethereum_0xABC", q.args[1])
        assertEquals("ethereum_0xABC", q.args[2])
    }

    @Test
    fun stateFilter_buildsEqualityWithEnumName() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.State(TransactionState.Pending)),
        )
        assertTrue(q.sql.contains("state = ?"))
        assertEquals("Pending", q.args[0])
    }

    @Test
    fun multipleFilters_areAndCombinedInOrder() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(
                TransactionsRequestFilter.Chains(listOf(Chain.Ethereum)),
                TransactionsRequestFilter.Types(listOf(TransactionType.Transfer)),
                TransactionsRequestFilter.AssetRankGreaterThan(15),
            ),
        )
        val whereStart = q.sql.indexOf("WHERE ")
        val orderStart = q.sql.indexOf("ORDER BY")
        val whereClause = q.sql.substring(whereStart, orderStart)
        assertEquals(2, " AND ".toRegex().findAll(whereClause).count())
        assertEquals("Ethereum", q.args[0])
        assertEquals("Transfer", q.args[1])
        assertEquals(15, q.args[2])
        assertEquals(DEFAULT_TRANSACTIONS_LIMIT, q.args[3])
    }
}
