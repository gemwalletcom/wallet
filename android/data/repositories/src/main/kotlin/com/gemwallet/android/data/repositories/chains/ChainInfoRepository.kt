package com.gemwallet.android.data.repositories.chains

import com.gemwallet.android.ext.available
import com.wallet.core.primitives.Chain
import uniffi.gemstone.assetDefaultRank
import javax.inject.Inject

class ChainInfoRepository @Inject constructor() {
    fun getAll() = Chain.available().sortedByDescending { assetDefaultRank(it.string) }
}