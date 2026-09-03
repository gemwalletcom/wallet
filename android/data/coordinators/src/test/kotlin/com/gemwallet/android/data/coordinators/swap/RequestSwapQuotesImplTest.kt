package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.swap.cases.matches
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
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
import uniffi.gemstone.GemSlippageCheck
import uniffi.gemstone.GemSwapQuoteServiceInterface
import uniffi.gemstone.SwapperSlippage
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemSwapTransfer
import uniffi.gemstone.SwapperAssetList
import uniffi.gemstone.SwapperProvider
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
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
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
        requestParams.value = quoteRequestParams(BigDecimal("12"))
        delay(700)
        job.cancelAndJoin()

        assertEquals(1, results.filterNotNull().size)
    }

    @Test
    fun `invalid input clears quote and late success is ignored`() = runBlocking {
        val fakeQuotes = StubSwapService(nonCancellableOnFirst = true)
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
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
        val integerKey = quoteRequestParams(BigDecimal("1")).key
        val decimalKey = quoteRequestParams(BigDecimal("1.0")).key

        assertEquals(integerKey, decimalKey)
        assertEquals(integerKey.hashCode(), decimalKey.hashCode())
    }

    @Test
    fun `quotes state matches numerically equal request values`() {
        val pay = assetInfo(symbol = "CAKE", tokenId = "cake")
        val receive = assetInfo(symbol = "BNB", tokenId = "bnb")
        val quotesState = SwapQuotesResult(
            requestKey = quoteRequestParams(BigDecimal("1")).key,
            pay = pay,
            receive = receive,
        )

        assertTrue(quotesState.matches(SwapQuoteRequestParams(BigDecimal("1.0"), pay, receive)))
    }

    @Test
    fun `successful quote refresh waits for the configured interval`() = runBlocking {
        val fakeQuotes = StubSwapService()
        val requester = RequestSwapQuotesImpl(fakeQuotes)
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))

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
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
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
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))

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
        val requestParams = MutableStateFlow<SwapQuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
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
        requestParams.value = quoteRequestParams(BigDecimal("2"))

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

    private fun assetInfo(symbol: String, tokenId: String): AssetInfo {
        val asset = Asset(
            id = AssetId(chain = Chain.SmartChain, tokenId = tokenId),
            name = symbol,
            symbol = symbol,
            decimals = 18,
            type = AssetType.TOKEN,
        )
        return mockAssetInfo(
            asset = asset,
            owner = Account(chain = Chain.SmartChain, address = "address", derivationPath = "m/44'/60'/0'/0/0"),
            balance = AssetBalance.create(asset, available = "100000000000000000000"),
        )
    }

    private fun quoteRequestParams(value: BigDecimal): SwapQuoteRequestParams {
        return SwapQuoteRequestParams(
            value = value,
            pay = assetInfo(symbol = "CAKE", tokenId = "cake"),
            receive = assetInfo(symbol = "BNB", tokenId = "bnb"),
        )
    }

    private class StubSwapService(
        private val shouldFail: Boolean = false,
        private val delayOnFirst: Long = 0,
        private val nonCancellableOnFirst: Boolean = false,
    ) : GemSwapQuoteServiceInterface {
        private val firstRequestStarted = CompletableDeferred<Unit>()
        var requestCount = 0

        override suspend fun getQuotes(
            fromAsset: uniffi.gemstone.Asset,
            toAsset: uniffi.gemstone.Asset,
            value: BigInteger,
            useMaxAmount: Boolean,
            slippageBps: UInt?,
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

        override suspend fun getTransfer(quote: SwapperQuote): GemSwapTransfer = throw UnsupportedOperationException()

        override fun supportedAssets(assetId: String): SwapperAssetList = SwapperAssetList(emptyList(), emptyList())

        override suspend fun suggestPair(payAssetId: String?): GemSwapPairSuggestion? = null

        override suspend fun addPrices(assetIds: List<String>) = Unit

        override fun currency(): String = "USD"

        override fun defaultSlippage(chain: String): SwapperSlippage = throw UnsupportedOperationException()

        override fun quoteDebounceMilliseconds(): ULong = 0u

        override fun selectedQuote(quotes: List<SwapperQuote>, preferred: SwapperProvider?): SwapperQuote? = throw UnsupportedOperationException()

        override fun refreshIntervalMilliseconds(): ULong = 0u

        override fun setSlippageBps(bps: UInt?) = Unit

        override fun slippageBps(): UInt? = null

        override fun slippageCheck(bps: UInt): GemSlippageCheck = throw UnsupportedOperationException()

        override suspend fun updateBalances(assetIds: List<String>) = Unit
    }
}
