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
        val query = buildExtendedTransactionsSql(filters = emptyList())
        assertTrue(query.sql.trimStart().startsWith("SELECT"))
        assertTrue(query.sql.contains("FROM transactions as tx"))
        assertTrue(query.sql.trimEnd().endsWith("ORDER BY tx.createdAt DESC LIMIT ?"))
        assertEquals(listOf<Any>(DEFAULT_TRANSACTIONS_LIMIT), query.args)
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
        val query = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Ethereum, Chain.Bitcoin))),
        )
        assertTrue(query.sql.contains("AND asset.chain IN (?,?)"))
        assertEquals(Chain.Ethereum.string, query.args[0])
        assertEquals(Chain.Bitcoin.string, query.args[1])
    }

    @Test
    fun typesFilter_buildsInClauseWithEnumNames() {
        val query = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Types(listOf(TransactionType.Transfer, TransactionType.Swap))),
        )
        assertTrue(query.sql.contains("AND tx.type IN (?,?)"))
        assertEquals("Transfer", query.args[0])
        assertEquals("Swap", query.args[1])
    }

    @Test
    fun assetRankGreaterThan_buildsInequalityOnJoinedAsset() {
        val query = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.AssetRankGreaterThan(15)),
        )
        assertTrue(query.sql.contains("AND asset.rank > ?"))
        assertEquals(15, query.args[0])
    }

    @Test
    fun assetFilter_matchesMainAndSwapAssets_bindsIdThreeTimes() {
        val assetId = AssetId(chain = Chain.Ethereum, tokenId = "0xABC")
        val query = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.Asset(assetId)),
        )
        assertTrue(
            query.sql.contains("(tx.assetId = ? OR swap.from_asset_id = ? OR swap.to_asset_id = ?)"),
        )
        assertEquals("ethereum_0xABC", query.args[0])
        assertEquals("ethereum_0xABC", query.args[1])
        assertEquals("ethereum_0xABC", query.args[2])
    }

    @Test
    fun stateFilter_buildsEqualityWithEnumName() {
        val query = buildExtendedTransactionsSql(
            filters = listOf(TransactionsRequestFilter.State(TransactionState.Pending)),
        )
        assertTrue(query.sql.contains("AND tx.state = ?"))
        assertEquals("Pending", query.args[0])
    }

    @Test
    fun multipleFilters_addOneAndPerFilter() {
        val baselineAndCount = " AND ".toRegex()
            .findAll(buildExtendedTransactionsSql(filters = emptyList()).sql).count()
        val query = buildExtendedTransactionsSql(
            filters = listOf(
                TransactionsRequestFilter.Chains(listOf(Chain.Ethereum)),
                TransactionsRequestFilter.Types(listOf(TransactionType.Transfer)),
                TransactionsRequestFilter.AssetRankGreaterThan(15),
            ),
        )
        val totalAndCount = " AND ".toRegex().findAll(query.sql).count()
        assertEquals(3, totalAndCount - baselineAndCount)
        assertEquals(Chain.Ethereum.string, query.args[0])
        assertEquals("Transfer", query.args[1])
        assertEquals(15, query.args[2])
        assertEquals(DEFAULT_TRANSACTIONS_LIMIT, query.args[3])
    }
}
