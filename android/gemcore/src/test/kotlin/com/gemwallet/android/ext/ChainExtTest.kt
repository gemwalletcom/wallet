package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.iconChain
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class ChainExtTest {
    @Test
    fun seiEvm_usesEvmMappings() {
        assertEquals(Chain.Sei, Chain.SeiEvm.iconChain())
    }

    @Test
    fun baseDrawsItsOwnLogo() {
        assertEquals(Chain.Base, Chain.Base.iconChain())
    }
}
