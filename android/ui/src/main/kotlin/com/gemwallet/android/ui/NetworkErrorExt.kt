package com.gemwallet.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.GemNetworkError

@Composable
fun GemNetworkError.localizedDescription(): String {
    return when (this) {
        GemNetworkError.Offline -> stringResource(
            R.string.errors_network_error,
            stringResource(R.string.errors_network_offline),
        )
        is GemNetworkError.Display -> message
        is GemNetworkError.Generic -> {
            if (message.isEmpty()) {
                stringResource(R.string.errors_error_occured)
            } else {
                stringResource(R.string.errors_network_error, message)
            }
        }
    }
}
