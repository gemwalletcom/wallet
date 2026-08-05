package com.gemwallet.android.blockchain.services

import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentProviderName
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentOptions
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.GemPreparedPayment

private const val EXPIRES_AT_SECONDS = 1_700_000_000L

class PaymentServiceTest {

    private val client = mockk<GemPaymentServiceInterface>()
    private val service = PaymentService(client)

    private fun gemQuote(assetId: String = "ethereum_0xtoken") = GemPaymentQuote(
        id = "option_1",
        paymentId = "pay_1",
        amount = GemPaymentAmount(assetId = assetId, value = "10", symbol = "USDT", decimals = 6),
        expiresAt = EXPIRES_AT_SECONDS,
        collectDataUrl = null,
        providerData = "{\"opaque\":true}",
    )

    private fun gemQuotes() = GemPaymentQuotes(
        merchant = GemPaymentMerchant(name = "Merchant", iconUrl = null),
        price = null,
        expiresAt = EXPIRES_AT_SECONDS,
        quotes = listOf(gemQuote()),
    )

    @Test
    fun getPaymentOptions_readsGatewaySecondsAsMillis() = runBlocking {
        coEvery { client.getPaymentOptions(any(), any()) } returns GemPaymentOptions.Quotes(gemQuotes())

        val options = service.getPaymentOptions(PaymentLink(PaymentProviderName.WalletConnectPay, "pay_1"), mockWallet())

        val quotes = (options as PaymentOptions.Quotes).content
        assertEquals(EXPIRES_AT_SECONDS * 1000, quotes.expiresAt)
        assertEquals(EXPIRES_AT_SECONDS * 1000, quotes.quotes.first().expiresAt)
        assertEquals(Chain.Ethereum, quotes.quotes.first().amount.assetId.chain)
        assertEquals("0xtoken", quotes.quotes.first().amount.assetId.tokenId)
    }

    @Test
    fun getPreparedPayment_returnsTheQuoteToTheGatewayUnchanged() = runBlocking {
        coEvery { client.getPaymentOptions(any(), any()) } returns GemPaymentOptions.Quotes(gemQuotes())
        val options = service.getPaymentOptions(PaymentLink(PaymentProviderName.WalletConnectPay, "pay_1"), mockWallet())
        val quotes = (options as PaymentOptions.Quotes).content

        val sentQuote = slot<GemPaymentQuote>()
        coEvery { client.getPreparedPayment(any(), any(), capture(sentQuote), any()) } returns GemPreparedPayment(
            quotes = gemQuotes(),
            quote = gemQuote(),
            actions = emptyList(),
            isRelayed = true,
        )

        val prepared = service.getPreparedPayment(
            PaymentProviderName.WalletConnectPay,
            quotes,
            quotes.quotes.first(),
            mockWallet(),
        )

        assertEquals(gemQuote(), sentQuote.captured)
        assertTrue(prepared.isRelayed)
    }
}
