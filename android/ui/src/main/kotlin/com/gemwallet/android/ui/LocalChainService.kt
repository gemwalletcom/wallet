package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemChainService

val LocalChainService = staticCompositionLocalOf<GemChainService> {
    error("LocalChainService is not provided")
}
