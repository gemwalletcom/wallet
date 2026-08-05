package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.model.PreparedPayment
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemApprovalData
import uniffi.gemstone.PaymentAction
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage

class ActivePaymentTest {

    @Test
    fun `results keep the gateway's action order`() {
        var payment = payment(listOf(approve(), signMessage()))

        payment = payment.completing("approval-hash")
        payment = payment.completing("signature")

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
            provider = PaymentProviderName.WalletConnectPay,
            quotes = quotes(),
            wallet = mockWallet(),
        ).prepared(PreparedPayment(quotes(), quote(), actions, isRelayed = true))

    private fun quotes() = PaymentQuotes(
        merchant = PaymentMerchant(name = "Gem Wallet Test Merchant", iconUrl = null),
        price = null,
        expiresAt = null,
        quotes = listOf(quote()),
    )

    private fun quote() = PaymentQuote(
        id = "opt_1",
        paymentId = "pay_1",
        amount = PaymentAmount(
            assetId = AssetId(Chain.Ethereum),
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
