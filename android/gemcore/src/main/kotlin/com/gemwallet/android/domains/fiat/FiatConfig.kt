package com.gemwallet.android.domains.fiat

import uniffi.gemstone.Config

object FiatConfig {
    private val config get() = Config().getFiatConfig()

    val insufficientNetworkFeeBuyAmount: Int get() = config.insufficientNetworkFeeBuyAmount
}
