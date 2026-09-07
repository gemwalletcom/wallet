package com.gemwallet.android.domains.fiat

import com.gemwallet.android.domains.gemConfig

object FiatConfig {
    private val config by lazy { gemConfig.getFiatConfig() }

    val insufficientNetworkFeeBuyAmount: Int get() = config.insufficientNetworkFeeBuyAmount
}
