package com.gemwallet.android.data.service.store.database

import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class ChainConvertersTest {
    private val converters = ChainConverters()

    @Test
    fun chains_useCanonicalIds() {
        Chain.entries.forEach { chain ->
            val stored = converters.fromChain(chain)

            assertEquals(chain.string, stored)
            assertEquals(chain, converters.toChain(stored))
        }
    }
}
