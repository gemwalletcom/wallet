package com.gemwallet.android.blockchain.services

import com.wallet.core.primitives.Chain
import org.junit.Assert.assertTrue
import org.junit.Test

class SignServiceTest {

    @Test
    fun `supports bitcoin chains`() {
        val service = SignService()

        assertTrue(service.supported(Chain.Bitcoin))
        assertTrue(service.supported(Chain.BitcoinCash))
        assertTrue(service.supported(Chain.Litecoin))
        assertTrue(service.supported(Chain.Doge))
        assertTrue(service.supported(Chain.Zcash))
    }
}
