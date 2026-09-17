package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemDeeplinkServiceInterface

val LocalDeeplinkService = staticCompositionLocalOf<GemDeeplinkServiceInterface> {
    error("LocalDeeplinkService is not provided")
}
