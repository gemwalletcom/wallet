package com.gemwallet.android.ui.components.list_item

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.WalletType

@get:StringRes
val WalletType.descriptionRes: Int get() = when (this) {
    WalletType.Multicoin, WalletType.Single -> R.string.common_secret_phrase
    WalletType.PrivateKey -> R.string.common_private_key
    WalletType.View -> R.string.common_address
}

fun WalletType.supportIcon(): String? = when (this) {
    WalletType.View -> "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}"
    else -> null
}
