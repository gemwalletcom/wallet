package com.gemwallet.android.domains.wallet_import

import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.words
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletImportType

fun ImportType.toGemImport(data: String): GemWalletImportType = when (walletType) {
    WalletType.Multicoin -> multicoinImport(data)
    WalletType.Single -> GemWalletImportType.SinglePhrase(words = data.words(), chain = importedChain().string)
    WalletType.View -> GemWalletImportType.Address(address = data, chain = importedChain().string)
    WalletType.PrivateKey -> GemWalletImportType.PrivateKey(value = data, chain = importedChain().string)
}

fun multicoinImport(phrase: String): GemWalletImportType = GemWalletImportType.MulticoinPhrase(
    words = phrase.words(),
    chains = Chain.entries.filter(Chain.available()::contains).map { it.string },
)

private fun ImportType.importedChain(): Chain = requireNotNull(chain) { "$walletType import requires a chain" }
