package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemAddressService

val LocalAddressService = staticCompositionLocalOf<GemAddressService> {
    error("LocalAddressService is not provided")
}
