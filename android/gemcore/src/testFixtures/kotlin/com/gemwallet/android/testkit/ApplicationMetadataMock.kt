package com.gemwallet.android.testkit

import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource

fun mockApplicationMetadata(name: String = "Uniswap", source: ApplicationMetadataSource = ApplicationMetadataSource.WalletConnect) = ApplicationMetadata(
    name = name,
    description = "Swap",
    url = "https://app.uniswap.org",
    icon = "https://app.uniswap.org/icon.png",
    source = source,
)
