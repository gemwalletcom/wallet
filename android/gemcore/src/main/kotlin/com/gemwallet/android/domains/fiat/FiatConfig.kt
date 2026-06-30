package com.gemwallet.android.domains.fiat

import uniffi.gemstone.Config

object FiatConfig {
    private val config get() = Config().getFiatConfig()

    val defaultBuyAmount: Int get() = config.defaultBuyAmount

    val defaultSellAmount: Int get() = config.defaultSellAmount

    val minimumAmount: Int get() = config.minimumAmount

    val maximumAmount: Int get() = config.maximumAmount

    val randomMaxAmount: Int get() = config.randomMaxAmount

    val suggestedAmounts: List<Int> get() = config.suggestedAmounts

    val insufficientNetworkFeeBuyAmount: Int get() = config.insufficientNetworkFeeBuyAmount
}
