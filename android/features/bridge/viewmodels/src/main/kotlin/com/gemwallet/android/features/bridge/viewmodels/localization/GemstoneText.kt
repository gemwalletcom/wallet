package com.gemwallet.android.features.bridge.viewmodels.localization

import android.content.Context
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemWalletConnectFailure

internal fun GemWalletConnectFailure.text(context: Context): String = when (this) {
    GemWalletConnectFailure.MaliciousOrigin -> context.getString(R.string.errors_connections_malicious_origin)
    GemWalletConnectFailure.Expired -> context.getString(R.string.wallet_connect_request_expired)
    is GemWalletConnectFailure.Failed -> message
}
