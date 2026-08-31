package com.gemwallet.android.application.fiat.cases

import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetAssetPriceUsd {
    operator fun invoke(assetId: AssetId): Flow<Double?>
}
