package com.gemwallet.android.ui

import androidx.compose.runtime.staticCompositionLocalOf
import uniffi.gemstone.GemAddressServiceInterface

val LocalAddressService = staticCompositionLocalOf<GemAddressServiceInterface> {
    error("LocalAddressService is not provided")
}
