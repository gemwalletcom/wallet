package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceStore
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetAssetPriceUsdImpl(
    private val priceStore: GemstonePriceStore,
) : GetAssetPriceUsd {

    override fun invoke(assetId: AssetId): Flow<Double?> = priceStore.observeUsdPrice(assetId)
}
