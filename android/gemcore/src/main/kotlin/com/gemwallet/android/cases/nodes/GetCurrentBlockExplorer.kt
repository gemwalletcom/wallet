package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Chain

interface GetCurrentBlockExplorer {
    fun getCurrentBlockExplorer(chain: Chain): String

    fun getBlockExplorerInfo(transaction: Transaction): Pair<String, String>
}