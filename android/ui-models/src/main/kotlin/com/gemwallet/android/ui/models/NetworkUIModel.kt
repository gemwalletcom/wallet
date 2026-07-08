package com.gemwallet.android.ui.models

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ext.networkName
import com.wallet.core.primitives.Chain

interface NetworkUIModel {
    val name: String
    val icon: String
}

class ChainNetworkUIModel(val chain: Chain) : NetworkUIModel {
    override val name: String
        get() = chain.networkName()

    override val icon: String
        get() = chain.getIconUrl()
}
