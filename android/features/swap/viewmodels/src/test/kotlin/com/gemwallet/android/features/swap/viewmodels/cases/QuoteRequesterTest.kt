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

    @Test
    fun `canceled in flight quote request does not trigger error callback`() = runBlocking {
        val getSwapQuotes = FakeGetSwapQuotes()
        val requester = QuoteRequester(getSwapQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val refreshState = MutableStateFlow(0L)
        val results = mutableListOf<QuotesState?>()
        val errors = mutableListOf<Throwable>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshState = refreshState,
                onStart = {},
                onError = { errors += it },
            ).collect { results += it }
        }

        withTimeout(2_000) {
            getSwapQuotes.firstRequestStarted.await()
        }
        requestParams.value = quoteRequestParams(BigDecimal("12"))
        delay(700)
        job.cancelAndJoin()

        assertTrue(errors.isEmpty())
        assertEquals(1, results.filterNotNull().size)
    }

    @Test
    fun `invalid input clears quote and late success is ignored`() = runBlocking {
        val getSwapQuotes = NonCancellableGetSwapQuotes()
        val requester = QuoteRequester(getSwapQuotes)
        val requestParams = MutableStateFlow<QuoteRequestParams?>(quoteRequestParams(BigDecimal.ONE))
        val refreshState = MutableStateFlow(0L)
        val results = mutableListOf<QuotesState?>()
        val errors = mutableListOf<Throwable>()

        val job = launch {
            requester.requestQuotes(
                requestParams = requestParams,
                refreshState = refreshState,
                onStart = {},
                onError = { errors += it },
            ).collect { results += it }
        }

        withTimeout(2_000) {
            getSwapQuotes.firstRequestStarted.await()
        }
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

    private class FakeGetSwapQuotes : GetSwapQuotes {
        val firstRequestStarted = CompletableDeferred<Unit>()

        override suspend fun getQuotes(
            ownerAddress: String,
            destination: String,
            from: Asset,
            to: Asset,
            amount: String,
            useMaxAmount: Boolean,
        ): List<SwapperQuote> {
            if (!firstRequestStarted.isCompleted) {
                firstRequestStarted.complete(Unit)
                delay(5_000)
            }
            return emptyList()
        }
    }

    private class NonCancellableGetSwapQuotes : GetSwapQuotes {
        val firstRequestStarted = CompletableDeferred<Unit>()

        override suspend fun getQuotes(
            ownerAddress: String,
            destination: String,
            from: Asset,
            to: Asset,
            amount: String,
            useMaxAmount: Boolean,
        ): List<SwapperQuote> {
            if (!firstRequestStarted.isCompleted) {
                firstRequestStarted.complete(Unit)
                withContext(NonCancellable) {
                    delay(200)
                }
            }
            return emptyList()
        }
    }
}
