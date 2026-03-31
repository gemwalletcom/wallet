package com.gemwallet.android.ext

import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import uniffi.gemstone.GemWalletConnectionSessionAppMetadata
import uniffi.gemstone.walletConnectAppShortName

fun WalletConnectionSessionAppMetadata.toGem() = GemWalletConnectionSessionAppMetadata(
    name = name,
    description = description,
    url = url,
    icon = icon,
)

val WalletConnectionSessionAppMetadata.shortName: String
    get() = walletConnectAppShortName(toGem())
