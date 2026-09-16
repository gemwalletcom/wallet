package com.gemwallet.android.features.import_wallet.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemWalletImportKind

@StringRes
internal fun GemWalletImportKind.tabStringRes(): Int = when (this) {
    GemWalletImportKind.ADDRESS -> R.string.common_address
    GemWalletImportKind.PHRASE -> R.string.common_phrase
    GemWalletImportKind.PRIVATE_KEY -> R.string.common_private_key
}

@StringRes
internal fun GemWalletImportKind.fieldStringRes(): Int = when (this) {
    GemWalletImportKind.ADDRESS -> R.string.wallet_import_address_field
    GemWalletImportKind.PHRASE -> R.string.common_secret_phrase
    GemWalletImportKind.PRIVATE_KEY -> R.string.common_private_key
}
