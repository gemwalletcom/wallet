package com.gemwallet.android.model

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletImportKind

data class ImportType(
    val walletType: WalletType,
    val chain: Chain? = null,
) {
    val kind: GemWalletImportKind
        get() = when (walletType) {
            WalletType.Multicoin, WalletType.Single -> GemWalletImportKind.PHRASE
            WalletType.PrivateKey -> GemWalletImportKind.PRIVATE_KEY
            WalletType.View -> GemWalletImportKind.ADDRESS
        }
}

fun GemWalletImportKind.toWalletType(chain: Chain?): WalletType = when (this) {
    GemWalletImportKind.PHRASE -> if (chain == null) WalletType.Multicoin else WalletType.Single
    GemWalletImportKind.PRIVATE_KEY -> WalletType.PrivateKey
    GemWalletImportKind.ADDRESS -> WalletType.View
}
