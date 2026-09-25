package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.TransactionsQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemTransactionFilter

@RunWith(AndroidJUnit4::class)
class TransactionsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = TransactionsQuery(database.transactionsDao())
    private val wallet = mockWallet(id = "wallet-1")
    private val otherWallet = mockWallet(id = "wallet-2")
    private val bitcoin = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val spam = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xspam"), name = "Spam", symbol = "SPAM", decimals = 18, type = AssetType.ERC20)
    private val received = mockTransaction(
        assetId = bitcoin.id,
        id = mockTransactionId(chain = Chain.Bitcoin, hash = "received"),
        from = "bc1qsender",
        to = "bc1qrecipient",
        value = "123456789012345678901234567890",
        direction = TransactionDirection.Incoming,
        createdAt = 100,
    )
    private val swap = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "swap"), type = TransactionType.Swap, state = TransactionState.Pending, value = "42", createdAt = 300)
    private val airdrop = mockTransaction(assetId = spam.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "airdrop"), feeAssetId = ethereum.id, createdAt = 400)
    private val failed = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "failed"), state = TransactionState.Failed, value = "7", createdAt = 200)
    private val otherWalletSend = mockTransaction(assetId = bitcoin.id, id = mockTransactionId(chain = Chain.Bitcoin, hash = "other"), value = "9", createdAt = 500)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(wallet.toRecord())
        database.walletsDao().insert(otherWallet.toRecord())
        database.assetsDao().insert(listOf(bitcoin.toRecord().copy(rank = 100), ethereum.toRecord().copy(rank = 90), spam.toRecord().copy(rank = 5)))
        database.transactionsDao().insert(listOf(received, swap, airdrop, failed).toRecord(wallet.id) + listOf(otherWalletSend).toRecord(otherWallet.id))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theActivityListsTheWalletTransactionsNewestFirstWithoutLowRankAssets() = runBlocking(Dispatchers.IO) {
        val items = query(wallet.id, TransactionsQueryFilter.activityDefaults(), 1000).first()

        assertEquals(listOf(swap, failed, received).map { it.stored() }, items.map { it.transaction })
        assertEquals(listOf(ethereum, ethereum, bitcoin), items.map { it.asset })
    }

    @Test
    fun withoutFiltersEveryWalletTransactionIsListed() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(airdrop, swap, failed, received).map { it.stored() }, query(wallet.id, emptyList(), 1000).first().map { it.transaction })
    }

    @Test
    fun anotherWalletSeesOnlyItsOwnTransactions() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(otherWalletSend.stored()), query(otherWallet.id, TransactionsQueryFilter.activityDefaults(), 1000).first().map { it.transaction })
    }

    @Test
    fun theChainAndTypeFiltersNarrowTheActivity() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(received.stored()), query(wallet.id, TransactionsQueryFilter.activity(listOf(Chain.Bitcoin), emptyList()), 1000).first().map { it.transaction })
        assertEquals(listOf(swap.stored()), query(wallet.id, TransactionsQueryFilter.activity(emptyList(), listOf(GemTransactionFilter.SWAPS)), 1000).first().map { it.transaction })
        assertEquals(listOf(failed.stored(), received.stored()), query(wallet.id, TransactionsQueryFilter.activity(emptyList(), listOf(GemTransactionFilter.TRANSFERS)), 1000).first().map { it.transaction })
    }

    @Test
    fun theAssetFilterListsTheTransactionsOfThatAssetOnly() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(swap, failed).map { it.stored() }, query(wallet.id, listOf(TransactionsQueryFilter.Asset(ethereum.id)), 1000).first().map { it.transaction })
        assertEquals(listOf(airdrop.stored()), query(wallet.id, listOf(TransactionsQueryFilter.Asset(spam.id)), 1000).first().map { it.transaction })
    }

    @Test
    fun theAssetFilterIncludesTransactionsThatTouchTheAsset() = runBlocking(Dispatchers.IO) {
        database.transactionsDao().replaceTransactionAssets(mapOf(swap.id.identifier to listOf(ethereum.id.toIdentifier(), bitcoin.id.toIdentifier())))

        assertEquals(listOf(swap, received).map { it.stored() }, query(wallet.id, listOf(TransactionsQueryFilter.Asset(bitcoin.id)), 1000).first().map { it.transaction })
    }

    @Test
    fun theStateFilterKeepsOnlyThoseStates() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(swap.stored()), query(wallet.id, TransactionsQueryFilter.pendingActivity(), 1000).first().map { it.transaction })
        assertEquals(listOf(failed.stored()), query(wallet.id, listOf(TransactionsQueryFilter.States(listOf(TransactionState.Failed))), 1000).first().map { it.transaction })
    }

    @Test
    fun theLimitKeepsTheNewestTransactions() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(airdrop, swap).map { it.stored() }, query(wallet.id, emptyList(), 2).first().map { it.transaction })
    }

    @Test
    fun aConfirmedTransactionLeavesThePendingActivity() = runBlocking(Dispatchers.IO) {
        val pending = TransactionsQueryFilter.pendingActivity()
        assertEquals(listOf(swap.stored()), query(wallet.id, pending, 1000).first().map { it.transaction })

        database.transactionsDao().updateTransactionState(swap.id, wallet.id, TransactionState.Confirmed, fee = null, blockNumber = null, metadata = null, confirmationEtaSeconds = null)

        assertEquals(emptyList<Transaction>(), query(wallet.id, pending, 1000).first().map { it.transaction })
    }

    private fun Transaction.stored(): Transaction = copy(sequence = "", utxoInputs = emptyList(), utxoOutputs = emptyList())
}
