package com.gemwallet.android.features.import_wallet.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemWalletImportException

@Composable
internal fun GemWalletImportException.string(): String = when (this) {
    is GemWalletImportException.InvalidSecretPhraseWords -> stringResource(R.string.errors_import_invalid_secret_phrase_word, words.joinToString())
    is GemWalletImportException.InvalidSecretPhrase -> stringResource(R.string.errors_import_invalid_secret_phrase)
    is GemWalletImportException.InvalidAddress -> stringResource(R.string.errors_invalid_address_name)
    is GemWalletImportException.InvalidPrivateKey -> stringResource(R.string.errors_import_invalid_private_key)
    is GemWalletImportException.MissingChain -> stringResource(R.string.errors_unknown)
}
