package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemNavigationServiceInterface

val LocalNavigationService = staticCompositionLocalOf<GemNavigationServiceInterface> {
    error("LocalNavigationService is not provided")
}
