package com.gemwallet.android.domains.swap

import com.wallet.core.primitives.swap.SwapData
import uniffi.gemstone.SwapperProvider

val SwapData.providerId: SwapperProvider
    get() = SwapperProvider.entries.first { it.name.lowercase() == quote.providerData.provider.string }
