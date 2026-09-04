package com.gemwallet.android.domains.swap

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.swap.SwapData
import uniffi.gemstone.SwapProvider

val SwapData.providerId: SwapProvider
    get() = quote.providerData.provider.toGem()
