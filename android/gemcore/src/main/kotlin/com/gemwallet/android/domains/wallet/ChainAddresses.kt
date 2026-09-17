package com.gemwallet.android.domains.wallet

import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet

val Wallet.chainAddresses: List<ChainAddress>
    get() = accounts.map { ChainAddress(chain = it.chain, address = it.address) }
