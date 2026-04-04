package com.gemwallet.android.cases.name

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ResolveNameTest {

    private val resolveName = object : ResolveName {
        override suspend fun resolveName(name: String, chain: Chain): NameRecord? = null
    }

    @Test
    fun canResolveName_matchesIosValidation() {
        assertFalse(resolveName.canResolveName("gemcoder"))
        assertFalse(resolveName.canResolveName("gemcoder."))
        assertTrue(resolveName.canResolveName("gemcoder.eth"))
    }
}
