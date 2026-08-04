package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.testkit.mockWallet
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemApprovalData
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.PaymentAction
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage

class ActivePaymentTest {

    @Test
    fun `signs before it spends`() {
        val payment = payment(listOf(approve(), signMessage()))

        assertEquals(listOf(1, 0), payment.order)
    }

    @Test
    fun `results keep the gateway's positions while signing first`() {
        var payment = payment(listOf(approve(), signMessage()))

        payment = payment.completing("signature")
        payment = payment.completing("approval-hash")

        assertEquals(listOf("approval-hash", "signature"), payment.results)
        assertNull(payment.step)
    }

    @Test
    fun `completing past the last action is a no-op`() {
        var payment = payment(listOf(signMessage()))
        payment = payment.completing("signature")

        val settled = payment.completing("duplicate")

        assertEquals(listOf("signature"), settled.results)
        assertEquals(1, settled.completed)
    }

    private fun payment(actions: List<PaymentAction>): ActivePayment =
        ActivePayment(
            provider = GemPaymentProviderName.WALLET_CONNECT_PAY,
            quotes = quotes(),
            wallet = mockWallet(),
        ).prepared(quote(), actions)

    private fun quotes() = GemPaymentQuotes(
        merchant = GemPaymentMerchant(name = "Gem Wallet Test Merchant", iconUrl = null),
        price = null,
        expiresAt = null,
        quotes = listOf(quote()),
    )

    private fun quote() = GemPaymentQuote(
        id = "opt_1",
        paymentId = "pay_1",
        amount = GemPaymentAmount(
            assetId = "ethereum",
            value = "1",
            symbol = "USDT",
            decimals = 6,
        ),
        expiresAt = null,
        collectDataUrl = null,
        providerData = "",
    )

    private fun signMessage() = PaymentAction.SignMessage(
        SignMessage(chain = "ethereum", signType = SignDigestType.EIP712, data = ByteArray(0)),
    )

    private fun approve() = PaymentAction.ApproveToken(
        chain = "ethereum",
        approval = GemApprovalData(token = "0xtoken", spender = "0xspender", value = "1", isUnlimited = true),
    )
}
