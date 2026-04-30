package com.gemwallet.android.data.service.store.database

import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionsQueryBuilderTest {

    @Test
    fun emptyFilters_baseQueryHasNoExtraConditions() {
        val q = buildExtendedTransactionsSql(filters = emptyList())
        assertTrue(q.sql.trimStart().startsWith("SELECT"))
        assertTrue(q.sql.contains("FROM transactions as tx"))
        assertTrue(q.sql.trimEnd().endsWith("ORDER BY tx.createdAt DESC LIMIT ?"))
        assertEquals(listOf<Any>(DEFAULT_TRANSACTIONS_LIMIT), q.args)
    }

    @Test
    fun emptyChainsOrTypes_areNoOps() {
        val baseline = buildExtendedTransactionsSql(filters = emptyList()).sql
        val chainsOnly = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Chains(emptyList())),
        ).sql
        val typesOnly = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Types(emptyList())),
        ).sql
        assertEquals(baseline, chainsOnly)
        assertEquals(baseline, typesOnly)
    }

    @Test
    fun chainsFilter_buildsInClauseOnJoinedAsset() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Ethereum, Chain.Bitcoin))),
        )
        assertTrue(q.sql.contains("AND asset.chain IN (?,?)"))
        assertEquals("Ethereum", q.args[0])
        assertEquals("Bitcoin", q.args[1])
    }

    @Test
    fun typesFilter_buildsInClauseWithEnumNames() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Types(listOf(TransactionType.Transfer, TransactionType.Swap))),
        )
        assertTrue(q.sql.contains("AND tx.type IN (?,?)"))
        assertEquals("Transfer", q.args[0])
        assertEquals("Swap", q.args[1])
    }

    @Test
    fun assetRankGreaterThan_buildsInequalityOnJoinedAsset() {
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.AssetRankGreaterThan(15)),
        )
        assertTrue(q.sql.contains("AND asset.rank > ?"))
        assertEquals(15, q.args[0])
    }

    @Test
    fun assetFilter_matchesMainAndSwapAssets_bindsIdThreeTimes() {
        val assetId = AssetId(chain = Chain.Ethereum, tokenId = "0xABC")
        val q = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Asset(assetId)),
        )
        assertTrue(
            q.sql.contains("(tx.assetId = ? OR swap.from_asset_id = ? OR swap.to_asset_id = ?)"),
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
        assertTrue(q.sql.contains("AND tx.state = ?"))
        assertEquals("Pending", q.args[0])
    }

    @Test
    fun multipleFilters_addOneAndPerFilter() {
        val baselineAndCount = " AND ".toRegex()
            .findAll(buildExtendedTransactionsSql(filters = emptyList()).sql).count()
        val q = buildExtendedTransactionsSql(
            filters = listOf(
                TransactionsRequestFilter.Chains(listOf(Chain.Ethereum)),
                TransactionsRequestFilter.Types(listOf(TransactionType.Transfer)),
                TransactionsRequestFilter.AssetRankGreaterThan(15),
            ),
        )
        val totalAndCount = " AND ".toRegex().findAll(q.sql).count()
        assertEquals(3, totalAndCount - baselineAndCount)
        assertEquals("Ethereum", q.args[0])
        assertEquals("Transfer", q.args[1])
        assertEquals(15, q.args[2])
        assertEquals(DEFAULT_TRANSACTIONS_LIMIT, q.args[3])
    }
}
