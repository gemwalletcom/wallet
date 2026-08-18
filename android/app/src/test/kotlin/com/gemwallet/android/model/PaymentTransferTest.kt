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
        val address = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"
        every { Chain.Xrp.checksumAddress(address) } returns address
        every { Chain.Xrp.isValidAddress(address) } returns true
        every { Chain.Xrp.isMemoSupport() } returns true
        val request = PaymentRequest(
            address = address,
            amount = PaymentAmount.ExactValue("10"),
            memo = "12345",
            assetId = AssetId(Chain.Xrp),
        )

        val destination = PaymentTransfer(mockAssetInfo(asset = mockAssetXrp())).destination(request)

        assertTrue(destination is PaymentDestination.Confirm)
        val params = (destination as PaymentDestination.Confirm).params
        assertEquals(BigInteger("10000000"), params.amount)
        assertEquals(address, params.destination()?.address)
        assertEquals("12345", params.memo())
    }

    @Test
    fun destination_withAmountWithoutMemo_requiresRecipient() {
        val address = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"
        every { Chain.Xrp.checksumAddress(address) } returns address
        every { Chain.Xrp.isValidAddress(address) } returns true
        every { Chain.Xrp.isMemoSupport() } returns true
        val request = PaymentRequest(
            address = address,
            amount = PaymentAmount.ExactValue("10"),
            memo = null,
            assetId = AssetId(Chain.Xrp),
        )

        val destination = PaymentTransfer(mockAssetInfo(asset = mockAssetXrp())).destination(request)

        assertTrue(destination is PaymentDestination.Recipient)
        val recipient = destination as PaymentDestination.Recipient
        assertEquals("10", (recipient.request.amount as PaymentAmount.ExactValue).content)
        assertEquals(null, recipient.request.memo)
    }
}
