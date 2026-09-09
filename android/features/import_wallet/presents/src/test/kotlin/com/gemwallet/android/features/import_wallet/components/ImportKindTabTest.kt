package com.gemwallet.android.features.import_wallet.components

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemWalletImportKind

class ImportKindTabTest {

    @Test
    fun selectedTabIndexFallsBackToTheFirstTab() {
        val tabs = listOf(GemWalletImportKind.PHRASE, GemWalletImportKind.ADDRESS)
        assertEquals(1, importTypeTabIndex(GemWalletImportKind.ADDRESS, tabs))
        assertEquals(0, importTypeTabIndex(GemWalletImportKind.PRIVATE_KEY, tabs))
    }
}
