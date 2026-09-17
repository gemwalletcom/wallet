package com.gemwallet.android.model

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemWalletImportKind

data class ImportType(
    val kind: GemWalletImportKind,
    val chain: Chain? = null,
) {
    companion object {
        fun phrase(chain: Chain? = null): ImportType = ImportType(GemWalletImportKind.PHRASE, chain)
    }
}
