package com.gemwallet.android.features.bridge.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemConnectionDetailRow

@StringRes
internal fun GemConnectionDetailRow.stringRes(): Int = when (this) {
    GemConnectionDetailRow.WALLET -> R.string.common_wallet
    GemConnectionDetailRow.DATE -> R.string.transaction_date
}
