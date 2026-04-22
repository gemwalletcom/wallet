package com.gemwallet.android.data.service.store.database

import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class NftChainConvertersTest {
    private val converters = NftChainConverters()

    @Test
    fun chain_roundTripAsAssetId() {
        assertEquals(Chain.Ethereum, converters.toChain(converters.fromChain(Chain.Ethereum)))
    }
}
