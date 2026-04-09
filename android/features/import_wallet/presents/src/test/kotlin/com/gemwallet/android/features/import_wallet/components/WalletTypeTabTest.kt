package com.gemwallet.android.features.import_wallet.components

import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletTypeTabTest {

    @Test
    fun nullChain_returnsPhaseAndAddress() {
        val tabs = importWalletTabs(null)
        assertEquals(listOf(WalletType.Single, WalletType.View), tabs)
    }
}
