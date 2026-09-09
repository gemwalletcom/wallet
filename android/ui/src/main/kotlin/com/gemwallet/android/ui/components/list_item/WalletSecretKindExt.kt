package com.gemwallet.android.ui.components.list_item

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemWalletSecretKind

@get:StringRes
val GemWalletSecretKind.titleRes: Int get() = when (this) {
    GemWalletSecretKind.PHRASE -> R.string.common_secret_phrase
    GemWalletSecretKind.PRIVATE_KEY -> R.string.common_private_key
}
