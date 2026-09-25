package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockGemSwapSession
import com.gemwallet.android.testkit.mockSwapQuoteRequestParams
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
import uniffi.gemstone.GemSwapPairFailure
import uniffi.gemstone.GemSwapPairSelection
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemSwapQuoteServiceInterface
import uniffi.gemstone.GemSwapSession
import uniffi.gemstone.GemSwapSide
import uniffi.gemstone.GemSwapTransfer
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapperQuote
import java.math.BigDecimal
import java.math.BigInteger

class RequestSwapQuotesImplTest {

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(true)

    @Test
    fun `canceled in flight quote request does not emit an error result`() = runBlocking {
        val fakeQuotes = StubSwapService(delayOnFirst = 5_000)
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())
        val results = mutableListOf<SwapQuotesResult?>()

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 30_000,
                debounceMillis = 500,
            ).collect { results += it }
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        requestParams.value = mockSwapQuoteRequestParams(BigDecimal("12"))
        delay(700)
        job.cancelAndJoin()

        assertEquals(1, results.filterNotNull().size)
    }

    @Test
    fun `invalid input clears quote and late success is ignored`() = runBlocking {
        val fakeQuotes = StubSwapService(nonCancellableOnFirst = true)
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())
        val results = mutableListOf<SwapQuotesResult?>()

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 30_000,
                debounceMillis = 500,
            ).collect { results += it }
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        requestParams.value = null
        delay(300)
        job.cancelAndJoin()

        assertEquals(listOf(null), results)
    }

    @Test
    fun `quote request key ignores BigDecimal scale`() {
        val integerKey = mockSwapQuoteRequestParams(BigDecimal("1")).key
        val decimalKey = mockSwapQuoteRequestParams(BigDecimal("1.0")).key

        assertEquals(integerKey, decimalKey)
        assertEquals(integerKey.hashCode(), decimalKey.hashCode())
    }

    @Test
    fun `successful quote refresh waits for the configured interval`() = runBlocking {
        val fakeQuotes = StubSwapService()
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 100,
                debounceMillis = 500,
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
        val fakeQuotes = StubSwapService(shouldFail = true)
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())
        val results = mutableListOf<SwapQuotesResult?>()

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 100,
                debounceMillis = 500,
            ).collect { results += it }
        }

        awaitCondition { fakeQuotes.requestCount >= 1 }
        delay(200)
        job.cancelAndJoin()

        assertEquals(1, fakeQuotes.requestCount)
        assertEquals(1, results.filterNotNull().size)
        assertTrue(results.filterNotNull().single().err != null)
    }

    @Test
    fun `automatic refresh stops in background and resumes in foreground`() = runBlocking {
        val fakeQuotes = StubSwapService()
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 100,
                debounceMillis = 500,
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
        val fakeQuotes = StubSwapService()
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(null)
        val results = mutableListOf<SwapQuotesResult?>()

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 30_000,
                debounceMillis = 500,
            ).collect { results += it }
        }

        delay(100)
        job.cancelAndJoin()

        assertEquals(listOf(null), results)
        assertEquals(0, fakeQuotes.requestCount)
    }

    @Test
    fun `changing params during debounce does not emit stale result`() = runBlocking {
        val fakeQuotes = StubSwapService()
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(mockSwapQuoteRequestParams())
        val results = mutableListOf<SwapQuotesResult?>()

        val job = launch {
            requester.invoke(
                requestParams = requestParams,
                refreshRequests = refreshRequests,
                refreshEnabled = refreshEnabled,
                onFetchStarted = {},
                refreshIntervalMillis = 30_000,
                debounceMillis = 500,
            ).collect { results += it }
        }

        delay(200)
        assertEquals(0, fakeQuotes.requestCount)
        requestParams.value = mockSwapQuoteRequestParams(BigDecimal("2"))

        awaitCondition { fakeQuotes.requestCount >= 1 }
        delay(100)
        job.cancelAndJoin()

        assertEquals(1, results.size)
    }

    private suspend fun awaitCondition(timeoutMs: Long = 2_000, condition: () -> Boolean) {
        withTimeout(timeoutMs) {
            while (!condition()) {
                delay(10)
            }
        }
    }

    private class StubSwapService(private val shouldFail: Boolean = false, private val delayOnFirst: Long = 0, private val nonCancellableOnFirst: Boolean = false) : GemSwapQuoteServiceInterface {
        private val firstRequestStarted = CompletableDeferred<Unit>()
        var requestCount = 0

        override fun newSession(): GemSwapSession = mockGemSwapSession()

        override suspend fun getQuotes(fromAsset: uniffi.gemstone.Asset, toAsset: uniffi.gemstone.Asset, value: BigInteger, useMaxAmount: Boolean, slippageBps: UInt?): List<SwapperQuote> {
            requestCount += 1

            if (!firstRequestStarted.isCompleted) {
                firstRequestStarted.complete(Unit)
                if (delayOnFirst > 0) delay(delayOnFirst)
                if (nonCancellableOnFirst) withContext(NonCancellable) { delay(200) }
            }

            if (shouldFail) throw SwapperException.ComputeQuoteException("boom")
            return emptyList()
        }

        override suspend fun getTransfer(quote: SwapperQuote): GemSwapTransfer = throw UnsupportedOperationException()

        override suspend fun suggestPair(payAssetId: String?): GemSwapPairSuggestion? = null

        override fun getCurrency(): uniffi.gemstone.Currency = com.wallet.core.primitives.Currency.USD.toGem()

        override fun setSlippageBps(bps: UInt?) = Unit

        override fun slippageBps(): UInt? = null

        override fun amountForPercent(available: java.math.BigInteger, percent: UInt): java.math.BigInteger = available * percent.toInt().toBigInteger() / java.math.BigInteger.valueOf(100)

        override fun selectPairAsset(selection: GemSwapPairSelection, side: GemSwapSide, assetId: String): GemSwapPairSelection = throw UnsupportedOperationException()

        override fun slippagePercent(bps: UInt): Double = throw UnsupportedOperationException()

        override suspend fun refreshPair(assetIds: List<String>): List<GemSwapPairFailure> = emptyList()
    }
}
