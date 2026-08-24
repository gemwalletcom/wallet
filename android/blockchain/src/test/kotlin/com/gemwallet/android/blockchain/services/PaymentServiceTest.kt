package com.gemwallet.android.blockchain.services

import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentAction
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemPaymentAction
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentOptions
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuoteData
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.GemPaymentServiceInterface

class PaymentServiceTest {

    private val client = mockk<GemPaymentServiceInterface>()
    private val service = PaymentService(client)

    private fun gemQuote() = GemPaymentQuote(
        id = "option_1",
        link = GemPaymentLink.WalletConnectPay("pay_1"),
        assetId = "ethereum",
        value = "14192816625800",
        collectDataUrl = null,
        providerData = "{\"opaque\":true}",
    )

    private fun gemQuotes() = GemPaymentQuotes(
        merchant = GemPaymentMerchant(name = "Merchant", iconUrl = null),
        price = null,
        quotes = listOf(gemQuote()),
    )

    @Test
    fun getOptions_mapsGatewayQuotesToWalletAssets() = runBlocking {
        coEvery { client.getOptions(any(), any()) } returns GemPaymentOptions.Quotes(gemQuotes())

        val options = service.getOptions(PaymentLink.WalletConnectPay("pay_1"), mockWallet())

        val quotes = (options as PaymentOptions.Quotes).content
        assertEquals("Merchant", quotes.merchant.name)
        assertEquals(Chain.Ethereum, quotes.quotes.first().assetId.chain)
        assertNull(quotes.quotes.first().assetId.tokenId)
    }

    @Test
    fun getQuoteData_returnsTheQuoteToTheGatewayUnchanged() = runBlocking {
        coEvery { client.getOptions(any(), any()) } returns GemPaymentOptions.Quotes(gemQuotes())
        val sent = slot<GemPaymentQuote>()
        coEvery { client.getQuoteData(capture(sent), any()) } returns GemPaymentQuoteData(
            quote = gemQuote(),
            action = GemPaymentAction.Send(
                chain = "ethereum",
                recipient = "0x57b2b4288220005234c0e88a04a7943193971d21",
                value = "14192816625800",
                data = "0xd3906488",
            ),
        )

        val options = service.getOptions(PaymentLink.WalletConnectPay("pay_1"), mockWallet())
        val quote = (options as PaymentOptions.Quotes).content.quotes.first()
        val quoteData = service.getQuoteData(quote, mockWallet())

        assertEquals(gemQuote(), sent.captured)
        val action = quoteData.action as PaymentAction.Send
        assertEquals(Chain.Ethereum, action.content.chain)
        assertEquals("14192816625800", action.content.value)
        assertEquals("0xd3906488", action.content.data)
    }
}
