package com.gemwallet.android.features.swap.viewmodels.cases

import com.gemwallet.android.cases.swap.GetSwapQuotes
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.QuotesState
import com.gemwallet.android.features.swap.viewmodels.models.matches
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.SwapperQuote
import java.math.BigDecimal

class QuoteRequesterTest {

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(true)

    @Test
    fun `canceled in flight quote request does not trigger error callback`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes(delayOnFirst = 5_000)
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val results = mutableListOf<QuotesState?>()
        val errors = mutableListOf<Throwable>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = { errors += it },
            ).collect { results += it }
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        requestParams.value = quoteRequestParams(BigDecimal("12"))
        delay(700)
        job.cancelAndJoin()

        assertTrue(errors.isEmpty())
        assertEquals(1, results.filterNotNull().size)
    }

    @Test
    fun `invalid input clears quote and late success is ignored`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes(nonCancellableOnFirst = true)
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val results = mutableListOf<QuotesState?>()
        val errors = mutableListOf<Throwable>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = { errors += it },
            ).collect { results += it }
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        requestParams.value = null
        delay(300)
        job.cancelAndJoin()

        assertTrue(errors.isEmpty())
        assertEquals(listOf(null), results)
    }

    @Test
    fun `quote request key ignores BigDecimal scale`() {
        val integerKey = quoteRequestParams(BigDecimal("1")).key
        val decimalKey = quoteRequestParams(BigDecimal("1.0")).key

        assertEquals(integerKey, decimalKey)
        assertEquals(integerKey.hashCode(), decimalKey.hashCode())
    }

    @Test
    fun `quotes state matches numerically equal request values`() {
        val pay = assetInfo(symbol = "CAKE", tokenId = "cake")
        val receive = assetInfo(symbol = "BNB", tokenId = "bnb")
        val quotesState = QuotesState(
            requestKey = quoteRequestParams(BigDecimal("1")).key,
            pay = pay,
            receive = receive,
        )

        assertTrue(quotesState.matches(QuoteRequestParams(BigDecimal("1.0"), pay, receive)))
    }

    @Test
    fun `successful quote refresh waits for the configured interval`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes()
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = {},
                refreshIntervalMillis = 100,
            ).collect()
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        delay(70)
        assertEquals(1, fakeQuotes.requestCount)
        awaitCondition { fakeQuotes.requestCount >= 2 }

        job.cancelAndJoin()
    }

    @Test
    fun `quote errors do not schedule automatic retries`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes(shouldFail = true)
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val errors = mutableListOf<Throwable>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = { errors += it },
                refreshIntervalMillis = 100,
            ).collect()
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        delay(200)
        job.cancelAndJoin()

        assertEquals(1, fakeQuotes.requestCount)
        assertEquals(1, errors.size)
    }

    @Test
    fun `automatic refresh stops in background and resumes in foreground`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes()
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = {},
                refreshIntervalMillis = 100,
            ).collect()
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }

        refreshEnabled.value = false
        delay(200)
        assertEquals(1, fakeQuotes.requestCount)

        refreshEnabled.value = true
        awaitCondition { fakeQuotes.requestCount >= 2 }

        job.cancelAndJoin()
    }

    @Test
    fun `null params emits null without calling quotes service`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes()
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(null)
        val results = mutableListOf<QuotesState?>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = {},
            ).collect { results += it }
        }

        delay(100)
        job.cancelAndJoin()

        assertEquals(listOf(null), results)
        assertEquals(0, fakeQuotes.requestCount)
    }

    @Test
    fun `changing params during debounce does not emit stale result`() = runBlocking {
        val fakeQuotes = StubGetSwapQuotes()
        val requester = QuoteRequester(fakeQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val results = mutableListOf<QuotesState?>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onError = {},
            ).collect { results += it }
        }

        // Change params during the 500ms debounce — before first fetch completes
        delay(200)
        assertEquals(0, fakeQuotes.requestCount)
        requestParams.value = quoteRequestParams(BigDecimal("2"))

        // Wait for the second request to complete
        awaitCondition { fakeQuotes.requestCount >= 1 }
        delay(100)
        job.cancelAndJoin()

        // Only one result — no stale emission from the first cancelled debounce
        assertEquals(1, results.size)
    }

    private suspend fun awaitCondition(timeoutMs: Long = 2_000, condition: () -> Boolean) {
        withTimeout(timeoutMs) {
            while (!condition()) {
                delay(10)
            }
        }
    }

    private fun assetInfo(symbol: String, tokenId: String): AssetInfo {
        val asset = Asset(
            id = AssetId(chain = Chain.SmartChain, tokenId = tokenId),
            name = symbol,
            symbol = symbol,
            decimals = 18,
            type = AssetType.TOKEN,
        )
        return AssetInfo(
            owner = Account(chain = Chain.SmartChain, address = "address", derivationPath = "m/44'/60'/0'/0/0"),
            asset = asset,
            balance = AssetBalance.create(asset, available = "100000000000000000000"),
            walletId = "wallet-id",
            walletType = WalletType.View,
            walletName = "Wallet",
        )
    }

    private fun quoteRequestParams(value: BigDecimal): QuoteRequestParams {
        return QuoteRequestParams(
            value = value,
            pay = assetInfo(symbol = "CAKE", tokenId = "cake"),
            receive = assetInfo(symbol = "BNB", tokenId = "bnb"),
        )
    }

    private class StubGetSwapQuotes(
        private val shouldFail: Boolean = false,
        private val delayOnFirst: Long = 0,
        private val nonCancellableOnFirst: Boolean = false,
    ) : GetSwapQuotes {
        private val firstRequestStarted = CompletableDeferred<Unit>()
        var requestCount = 0

        override suspend fun getQuotes(
            ownerAddress: String,
            destination: String,
            from: Asset,
            to: Asset,
            amount: String,
            useMaxAmount: Boolean,
        ): List<SwapperQuote> {
            requestCount += 1

            if (!firstRequestStarted.isCompleted) {
                firstRequestStarted.complete(Unit)
                if (delayOnFirst > 0) delay(delayOnFirst)
                if (nonCancellableOnFirst) withContext(NonCancellable) { delay(200) }
            }

            if (shouldFail) throw IllegalStateException("boom")
            return emptyList()
        }
    }
}
