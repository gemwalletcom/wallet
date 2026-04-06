package com.gemwallet.android.features.import_wallet.components

import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletTypeTabTest {

    @Test
    fun privateKeyImport_usesPrivateKeyTabIndex() {
        assertEquals(1, importTypeTabIndex(WalletType.PrivateKey))
    }

    @Test
    fun viewImport_usesAddressTabIndex() {
        assertEquals(2, importTypeTabIndex(WalletType.View))
    }
}
