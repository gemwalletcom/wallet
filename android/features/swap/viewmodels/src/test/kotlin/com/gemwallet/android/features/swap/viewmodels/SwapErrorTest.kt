package com.gemwallet.android.features.swap.viewmodels

import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.toError
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.SwapperException

class SwapErrorTest {
    @Test
    fun unsupportedChainMapsToUnsupportedAsset() {
        assertTrue(SwapError.toError(SwapperException.NotSupportedChain()) is SwapError.NotSupportedAsset)
    }

    @Test
    fun quoteFailuresMapToNoQuote() {
        val errors = listOf(
            SwapperException.NoQuoteAvailable(),
            SwapperException.NoAvailableProvider(),
            SwapperException.InvalidRoute(),
            SwapperException.ComputeQuoteException("HTTP error: status 500"),
            SwapperException.TransactionException("failed to decode transaction"),
        )

        errors.forEach {
            assertTrue(SwapError.toError(it) is SwapError.NoQuote)
        }
    }
}
