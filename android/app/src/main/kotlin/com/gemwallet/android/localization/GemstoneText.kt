package com.gemwallet.android.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.ConnectionStatus
import com.wallet.core.primitives.WalletSource

@StringRes
internal fun ConnectionStatus.stringRes(): Int? = when (this) {
    ConnectionStatus.Online -> null
    ConnectionStatus.NoInternet -> R.string.errors_no_internet_connection
    ConnectionStatus.NoService -> R.string.errors_no_service_connection
}

@StringRes
internal fun WalletSource.stringRes(): Int = when (this) {
    WalletSource.Create -> R.string.wallet_new_title
    WalletSource.Import -> R.string.wallet_import_title
}
