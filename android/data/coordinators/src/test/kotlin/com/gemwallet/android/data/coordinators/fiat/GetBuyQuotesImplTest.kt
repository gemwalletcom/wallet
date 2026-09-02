package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.test.StandardTestDispatcher
import org.junit.After
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemFiatService

@OptIn(ExperimentalCoroutinesApi::class)
class GetBuyQuotesImplTest {

    private val mainDispatcher = StandardTestDispatcher()
    private val asset = mockAsset()
    private val callingThreads = mutableListOf<String>()

    private val fiatService = mockk<GemFiatService> {
        coEvery { getQuotes(any(), any(), any(), any(), any()) } answers {
            callingThreads += Thread.currentThread().name
            emptyList()
        }
    }
    private val subject = GetBuyQuotesImpl(fiatService)

    @Before
    fun setUp() = Dispatchers.setMain(mainDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `quotes are requested off the caller thread so the store callbacks never touch the database on main`() = runTest(mainDispatcher) {
        val callerThread = Thread.currentThread().name

        subject(
            walletId = WalletId("wallet-1"),
            asset = asset,
            type = FiatQuoteType.Buy,
            currency = Currency.USD,
            amount = 50.0,
        )

        val serviceThread = callingThreads.single()
        assertNotEquals(
            "getQuotes must not run on the caller's thread: GemFiatService calls back into GemWalletStore, " +
                "which reads Room synchronously and throws when that lands on the main thread",
            callerThread,
            serviceThread,
        )
        assertTrue(
            "expected an IO pool thread, got $serviceThread",
            serviceThread.startsWith("DefaultDispatcher-worker"),
        )
    }
}
