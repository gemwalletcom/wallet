package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.TransactionQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemTransactionDetailsServiceInterface
import uniffi.gemstone.BlockExplorerLink as GemBlockExplorerLink
import uniffi.gemstone.WalletType as GemWalletType

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionViewModelTest {

    private val dispatcher = UnconfinedTestDispatcher()
    private val transactionExtended = mockTransactionExtended(
        transaction = mockTransaction(assetId = mockAssetId(chain = Chain.Near), type = TransactionType.Swap),
        asset = mockAsset(id = mockAssetId(chain = Chain.Near)),
    )
    private val transactionId = transactionExtended.transaction.id
    private val wallet = mockWallet(id = WalletId("opened"), type = WalletType.View)
    private val service = mockk<GemTransactionDetailsServiceInterface>()
    private val transactionQuery = mockk<TransactionQuery>()
    private val walletQuery = mockk<WalletQuery> { every { this@mockk(wallet.id) } returns flowOf(wallet) }
    private val model by lazy {
        TransactionViewModel(
            walletQuery,
            transactionQuery,
            service,
            SavedStateHandle(mapOf(RouteArgument.WalletId.key to wallet.id.id, RouteArgument.TransactionId.key to transactionId.identifier)),
            dispatcher,
            mockk(relaxed = true),
        )
    }

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        model.viewModelScope.cancel()
        Dispatchers.resetMain()
    }

    @Test
    fun theDetailRowsComeFromCoreForTheStoredTransactionAndTheWalletType() {
        val rows = mockGemTransactionDetailRows(
            explorer = GemBlockExplorerLink("NEAR Intents", "https://explorer.near-intents.org/transactions/recipient-address"),
        )
        every { transactionQuery(wallet.id, transactionId) } returns flowOf(transactionExtended)
        every { service.detailRows(transactionExtended.toGem(), GemWalletType.VIEW) } returns rows

        assertEquals(rows, model.data.value)
        assertEquals("NEAR Intents", model.data.value?.explorer?.name)
        assertEquals("https://explorer.near-intents.org/transactions/recipient-address", model.data.value?.explorer?.link)
    }

    @Test
    fun theDetailsReadTheWalletTheyWereOpenedForWithoutASession() {
        val rows = mockGemTransactionDetailRows()
        every { transactionQuery(wallet.id, transactionId) } returns flowOf(transactionExtended)
        every { service.detailRows(any(), any()) } returns rows

        assertEquals(rows, model.data.value)
    }

    @Test
    fun theDetailsClearWhenTheShownRecordIsDeleted() {
        val record = MutableStateFlow<TransactionExtended?>(transactionExtended)
        val rows = mockGemTransactionDetailRows()
        every { transactionQuery(wallet.id, transactionId) } returns record
        every { service.detailRows(any(), any()) } returns rows

        assertEquals(rows, model.data.value)

        record.value = null

        assertNull(model.data.value)
    }
}
