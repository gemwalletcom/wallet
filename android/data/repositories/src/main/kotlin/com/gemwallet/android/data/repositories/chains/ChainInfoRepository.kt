package com.gemwallet.android.data.repositories.chains

import com.gemwallet.android.domains.asset.defaultAssetRank
import com.wallet.core.primitives.Chain
import javax.inject.Inject

class ChainInfoRepository @Inject constructor() {
    fun getAll() = Chain.entries.sortedByDescending { it.defaultAssetRank }
}
