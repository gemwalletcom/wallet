package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemAssetsServiceInterface

val LocalAssetsService = staticCompositionLocalOf<GemAssetsServiceInterface> {
    error("LocalAssetsService is not provided")
}
