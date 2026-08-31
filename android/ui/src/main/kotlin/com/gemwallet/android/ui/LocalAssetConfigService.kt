package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemAssetConfigService

val LocalAssetConfigService = staticCompositionLocalOf<GemAssetConfigService> {
    error("LocalAssetConfigService is not provided")
}
