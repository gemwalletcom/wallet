package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain

interface GetNodeUrlCase {
    fun getNodeUrl(chain: Chain): String
}
