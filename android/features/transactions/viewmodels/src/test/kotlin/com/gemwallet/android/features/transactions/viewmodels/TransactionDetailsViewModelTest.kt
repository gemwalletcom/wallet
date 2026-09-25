package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.TransactionQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockSession
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
class TransactionDetailsViewModelTest {

    private val dispatcher = UnconfinedTestDispatcher()
    private val transactionExtended = mockTransactionExtended(
        transaction = mockTransaction(assetId = mockAssetId(chain = Chain.Near), type = TransactionType.Swap),
        asset = mockAsset(id = mockAssetId(chain = Chain.Near)),
    )
    private val transactionId = transactionExtended.transaction.id
    private val service = mockk<GemTransactionDetailsServiceInterface>()
    private val transactionQuery = mockk<TransactionQuery>()
    private val session = MutableStateFlow<Session?>(null)
    private val getSession = mockk<GetSession> { every { this@mockk.invoke() } returns session }
    private val model by lazy {
        TransactionDetailsViewModel(getSession, transactionQuery, service, SavedStateHandle(mapOf(RouteArgument.TransactionId.key to transactionId.identifier)), dispatcher, mockk(relaxed = true))
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
        val wallet = mockWallet(type = WalletType.View)
        val rows = mockGemTransactionDetailRows(
            transaction = transactionExtended,
            explorer = GemBlockExplorerLink("NEAR Intents", "https://explorer.near-intents.org/transactions/recipient-address"),
        )
        every { transactionQuery(wallet.id, transactionId) } returns flowOf(transactionExtended)
        every { service.detailRows(transactionExtended.toGem(), GemWalletType.VIEW) } returns rows

        session.value = mockSession(wallet = wallet)

        assertEquals(rows, model.data.value)
        assertEquals("NEAR Intents", model.data.value?.explorer?.name)
        assertEquals("https://explorer.near-intents.org/transactions/recipient-address", model.data.value?.explorer?.link)
    }

    @Test
    fun theDetailsClearWhenTheOtherWalletHasNoSuchRecord() {
        val first = mockWallet(id = WalletId("first"))
        val second = mockWallet(id = WalletId("second"))
        session.value = mockSession(wallet = first)
        val rows = mockGemTransactionDetailRows(transaction = transactionExtended)
        every { transactionQuery(first.id, transactionId) } returns flowOf(transactionExtended)
        every { transactionQuery(second.id, transactionId) } returns flowOf(null)
        every { service.detailRows(any(), any()) } returns rows

        assertEquals(rows, model.data.value)

        session.value = mockSession(wallet = second)

        assertNull(model.data.value)
    }

    @Test
    fun theDetailsClearWhenTheShownRecordIsDeleted() {
        val wallet = mockWallet()
        val record = MutableStateFlow<TransactionExtended?>(transactionExtended)
        val rows = mockGemTransactionDetailRows(transaction = transactionExtended)
        every { transactionQuery(wallet.id, transactionId) } returns record
        every { service.detailRows(any(), any()) } returns rows

        session.value = mockSession(wallet = wallet)
        assertEquals(rows, model.data.value)

        record.value = null

        assertNull(model.data.value)
    }

    @Test
    fun noSessionShowsNoDetails() {
        assertNull(model.data.value)
    }
}
