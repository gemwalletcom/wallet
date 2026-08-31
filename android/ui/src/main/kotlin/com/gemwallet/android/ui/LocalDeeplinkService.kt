package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemDeeplinkService

val LocalDeeplinkService = staticCompositionLocalOf<GemDeeplinkService> {
    error("LocalDeeplinkService is not provided")
}
