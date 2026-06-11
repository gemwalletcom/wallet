package com.gemwallet.android.domains.search

import uniffi.gemstone.Config

object WalletSearchConfig {
    private val config get() = Config().getWalletSearchConfig()

    val assetsInitialLimit: Int get() = config.assetsInitialLimit.toInt()

    val assetsTagLimit: Int get() = config.assetsTagLimit.toInt()

    val assetsSearchLimit: Int get() = config.assetsSearchLimit.toInt()

    val perpetualsPreviewLimit: Int get() = config.perpetualsPreviewLimit.toInt()

    val resultsLimit: Int get() = config.resultsLimit.toInt()
}
