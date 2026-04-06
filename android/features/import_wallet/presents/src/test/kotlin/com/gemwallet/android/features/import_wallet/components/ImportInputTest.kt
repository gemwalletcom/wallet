package com.gemwallet.android.features.import_wallet.components

import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ImportInputTest {

    @Test
    fun phraseImport_showsPhraseSuggestions() {
        assertTrue(supportsPhraseSuggestions(WalletType.Single))
    }

    @Test
    fun privateKeyImport_hidesPhraseSuggestions() {
        assertFalse(supportsPhraseSuggestions(WalletType.PrivateKey))
    }
}
