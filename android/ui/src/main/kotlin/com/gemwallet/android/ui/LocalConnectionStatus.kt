package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import com.wallet.core.primitives.ConnectionStatus
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

val LocalConnectionStatus = staticCompositionLocalOf<StateFlow<ConnectionStatus>> {
    MutableStateFlow(ConnectionStatus.Online)
}
