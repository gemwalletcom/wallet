package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.WalletType

internal fun WalletType.supportIcon(): String? = when (this) {
    WalletType.View -> "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}"
    else -> null
}
