package com.gemwallet.android.domains.confirm

import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSignerError

class ConfirmErrorTest {
    @Test
    fun signerErrorsMapToConfirmErrors() {
        val dust = GemSignerError.DustThreshold.toConfirmError(Chain.Bitcoin)
        val signingFailures = listOf(
            GemSignerError.InvalidInput("invalid transaction"),
            GemSignerError.SigningError("signing failed"),
            GemSignerError.InsufficientFunds,
            GemSignerError.SwapValueBelowMinimum,
        )

        assertTrue(dust is ConfirmError.DustThreshold)
        assertEquals(Chain.Bitcoin, (dust as ConfirmError.DustThreshold).chain)
        signingFailures.forEach { error ->
            assertSame(ConfirmError.SignFail, error.toConfirmError(Chain.Bitcoin))
        }
    }
}
