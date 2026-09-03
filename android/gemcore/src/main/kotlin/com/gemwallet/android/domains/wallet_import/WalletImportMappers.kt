package com.gemwallet.android.domains.wallet_import

import com.gemwallet.android.application.wallet_import.values.ImportError
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.words
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletImportException
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

fun GemWalletImportType.validatedOrImportError(): GemWalletImportType = try {
    validated()
} catch (error: GemWalletImportException) {
    throw error.toImportError()
}

fun GemWalletImportException.toImportError(): ImportError = when (this) {
    is GemWalletImportException.InvalidSecretPhraseWords -> ImportError.InvalidWords(words)
    is GemWalletImportException.InvalidSecretPhrase -> ImportError.InvalidationSecretPhrase
    is GemWalletImportException.InvalidPrivateKey -> ImportError.InvalidationPrivateKey
    is GemWalletImportException.InvalidAddress -> ImportError.InvalidAddress
}

private fun ImportType.importedChain(): Chain = requireNotNull(chain) { "$walletType import requires a chain" }
