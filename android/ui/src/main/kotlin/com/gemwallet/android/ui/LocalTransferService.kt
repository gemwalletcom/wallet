package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemTransferService

val LocalTransferService = staticCompositionLocalOf<GemTransferService> {
    error("LocalTransferService is not provided")
}
