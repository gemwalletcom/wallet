package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

interface GetFeeAssets {
    operator fun invoke(chain: Chain): Flow<List<AssetInfo>>
}
