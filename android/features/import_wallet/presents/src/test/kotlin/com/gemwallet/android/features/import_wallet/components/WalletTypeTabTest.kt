package com.gemwallet.android.features.import_wallet.components

import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletTypeTabTest {

    @Test
    fun selectedTabIndexFallsBackToTheFirstTab() {
        val tabs = listOf(WalletType.Single, WalletType.View)
        assertEquals(1, importTypeTabIndex(WalletType.View, tabs))
        assertEquals(0, importTypeTabIndex(WalletType.PrivateKey, tabs))
    }
}
