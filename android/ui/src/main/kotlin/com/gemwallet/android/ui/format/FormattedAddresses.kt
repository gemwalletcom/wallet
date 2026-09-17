package com.gemwallet.android.ui.format

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.style.formatShort
import com.wallet.core.primitives.Chain

@Composable
fun rememberFormattedAddress(
    address: String,
    chain: Chain? = null,
): String {
    val addressService = LocalAddressService.current
    return remember(addressService, address, chain) {
        addressService.formatShort(address, chain?.string)
    }
}
