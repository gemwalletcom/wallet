package com.gemwallet.android.features.bridge.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemWalletConnectException

@Composable
internal fun Throwable.walletConnectMessage(): String? = when (this) {
    is GemWalletConnectException.UnsupportedChains -> stringResource(R.string.errors_connections_unsupported_chain)
    is GemWalletConnectException.InvalidOrigin -> stringResource(R.string.errors_connections_malicious_origin)
    is GemWalletConnectException.UnsupportedWallets -> stringResource(R.string.errors_connections_no_supported_wallets)
    is GemWalletConnectException.Service -> msg
    else -> null
}
