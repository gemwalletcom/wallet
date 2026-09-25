package com.gemwallet.android.data.services.store.database

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionsQueryBuilderTest {

    private val walletId = WalletId("wallet-1")
    private val limit = 1000
    private val baseArgCount = 1 // walletId is the only bound arg in TRANSACTION_LIST_SOURCE

    @Test
    fun emptyFilters_baseQueryHasNoExtraConditions() {
        val query = buildTransactionListSql(walletId, limit = limit, filters = emptyList())
        assertTrue(query.sql.trimStart().startsWith("SELECT"))
        assertTrue(query.sql.contains("FROM transactions as tx"))
        assertTrue(query.sql.trimEnd().endsWith("ORDER BY tx.createdAt DESC LIMIT ?"))
        assertEquals(listOf<Any>(walletId.id, 1000), query.args)
    }

    @Test
    fun emptyChainsOrTypes_areNoOps() {
        val baseline = buildTransactionListSql(walletId, limit = limit, filters = emptyList()).sql
        val chainsOnly = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.Chains(emptyList())),
        ).sql
        val typesOnly = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.Types(emptyList())),
        ).sql
        assertEquals(baseline, chainsOnly)
        assertEquals(baseline, typesOnly)
    }

    @Test
    fun chainsFilter_buildsInClauseOnJoinedAsset() {
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Ethereum, Chain.Bitcoin))),
        )
        assertTrue(query.sql.contains("AND asset.chain IN (?,?)"))
        assertEquals(Chain.Ethereum.string, query.args[baseArgCount])
        assertEquals(Chain.Bitcoin.string, query.args[baseArgCount + 1])
    }

    @Test
    fun typesFilter_buildsInClauseWithEnumNames() {
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.Types(listOf(TransactionType.Transfer, TransactionType.Swap))),
        )
        assertTrue(query.sql.contains("AND tx.type IN (?,?)"))
        assertEquals("Transfer", query.args[baseArgCount])
        assertEquals("Swap", query.args[baseArgCount + 1])
    }

    @Test
    fun assetRankGreaterThan_buildsInequalityOnJoinedAsset() {
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.AssetRankGreaterThan(15)),
        )
        assertTrue(query.sql.contains("AND asset.rank > ?"))
        assertEquals(15, query.args[baseArgCount])
    }

    @Test
    fun assetFilter_matchesMainAndTransactionAssets_bindsIdTwice() {
        val assetId = AssetId(chain = Chain.Ethereum, tokenId = "0xABC")
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.Asset(assetId)),
        )
        assertTrue(
            query.sql.contains("(tx.assetId = ? OR EXISTS (SELECT 1 FROM transactions_assets AS ta WHERE ta.tx_id = tx.id AND ta.asset_id = ?))"),
        )
        assertEquals(listOf("ethereum_0xABC", "ethereum_0xABC"), query.args.drop(baseArgCount).take(2))
    }

    @Test
    fun statesFilter_buildsInClauseWithEnumNames() {
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(TransactionsRequestFilter.States(listOf(TransactionState.Pending, TransactionState.InTransit))),
        )
        assertTrue(query.sql.contains("AND tx.state IN (?,?)"))
        assertEquals("Pending", query.args[baseArgCount])
        assertEquals("InTransit", query.args[baseArgCount + 1])
    }

    @Test
    fun multipleFilters_addOneAndPerFilter() {
        val baselineAndCount = " AND ".toRegex()
            .findAll(buildTransactionListSql(walletId, limit = limit, filters = emptyList()).sql).count()
        val query = buildTransactionListSql(
            walletId,
            limit = limit,
            filters = listOf(
                TransactionsRequestFilter.Chains(listOf(Chain.Ethereum)),
                TransactionsRequestFilter.Types(listOf(TransactionType.Transfer)),
                TransactionsRequestFilter.AssetRankGreaterThan(15),
            ),
        )
        val totalAndCount = " AND ".toRegex().findAll(query.sql).count()
        assertEquals(3, totalAndCount - baselineAndCount)
        assertEquals(Chain.Ethereum.string, query.args[baseArgCount])
        assertEquals("Transfer", query.args[baseArgCount + 1])
        assertEquals(15, query.args[baseArgCount + 2])
    }

    @Test
    fun countSql_sharesFilterClausesWithExtendedSql() {
        val filters = listOf(
            TransactionsRequestFilter.AssetRankGreaterThan(15),
            TransactionsRequestFilter.States(listOf(TransactionState.Pending, TransactionState.InTransit)),
        )
        val query = buildTransactionsCountSql(walletId, filters)
        assertTrue(query.sql.trimStart().startsWith("SELECT COUNT(DISTINCT tx.id)"))
        assertTrue(query.sql.contains("AND asset.rank > ?"))
        assertTrue(query.sql.contains("AND tx.state IN (?,?)"))
        assertFalse(query.sql.contains("ORDER BY"))
        assertEquals(listOf<Any>(walletId.id, 15, "Pending", "InTransit"), query.args)
    }

    @Test
    fun theListAndTheCountReadNoPrices() {
        assertFalse(buildTransactionListSql(walletId, limit = limit, filters = emptyList()).sql.contains("prices"))
        assertFalse(buildTransactionsCountSql(walletId, filters = emptyList()).sql.contains("prices"))
    }

    @Test
    fun walletIdIsBoundOnce() {
        val query = buildTransactionListSql(walletId, limit = limit, filters = emptyList())
        assertEquals(walletId.id, query.args[0])
    }
}
