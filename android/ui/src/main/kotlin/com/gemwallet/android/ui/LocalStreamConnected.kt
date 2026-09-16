package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

val LocalStreamConnected = staticCompositionLocalOf<StateFlow<Boolean>> {
    MutableStateFlow(false)
}
