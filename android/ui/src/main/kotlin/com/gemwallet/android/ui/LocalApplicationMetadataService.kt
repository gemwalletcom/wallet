package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemApplicationMetadataService

val LocalApplicationMetadataService = staticCompositionLocalOf<GemApplicationMetadataService> {
    error("LocalApplicationMetadataService is not provided")
}
