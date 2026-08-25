package com.gemwallet.android.model

import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetXrp
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentRequest
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

class PaymentTransferTest {

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
    }

    @After
    fun tearDown() {
        unmockkStatic("com.gemwallet.android.ext.ChainKt")
    }

    @Test
    fun destination_withAmountAndMemo_confirms() {
        val request = paymentRequest(memo = "12345")

        val destination = PaymentTransfer(mockAssetInfo(asset = mockAssetXrp())).destination(request)

        assertTrue(destination is PaymentDestination.Confirm)
        val params = (destination as PaymentDestination.Confirm).params
        assertEquals(BigInteger("10000000"), params.amount)
        assertEquals(XRP_ADDRESS, params.destination()?.address)
        assertEquals("12345", params.memo())
        assertEquals(listOf("reference"), params.references)
    }

    @Test
    fun destination_withAmountWithoutMemo_requiresRecipient() {
        val request = paymentRequest(memo = null)

        val destination = PaymentTransfer(mockAssetInfo(asset = mockAssetXrp())).destination(request)

        assertTrue(destination is PaymentDestination.Recipient)
        val recipient = destination as PaymentDestination.Recipient
        assertEquals(PaymentAmount.ExactValue("10"), recipient.request.amount)
        assertEquals(null, recipient.request.memo)
    }

    private fun paymentRequest(memo: String?): PaymentRequest {
        every { Chain.Xrp.checksumAddress(XRP_ADDRESS) } returns XRP_ADDRESS
        every { Chain.Xrp.isValidAddress(XRP_ADDRESS) } returns true
        every { Chain.Xrp.isMemoSupport() } returns true
        return PaymentRequest(
            address = XRP_ADDRESS,
            amount = PaymentAmount.ExactValue("10"),
            memo = memo,
            references = listOf("reference"),
            assetId = AssetId(Chain.Xrp),
        )
    }

    private companion object {
        const val XRP_ADDRESS = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"
    }
}
