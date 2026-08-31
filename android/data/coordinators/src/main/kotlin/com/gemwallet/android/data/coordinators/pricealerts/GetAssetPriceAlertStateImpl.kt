package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetPriceAlertStateImpl(
    private val priceAlertStore: GemstonePriceAlertStore,
) : GetAssetPriceAlertState {

    override fun isAssetPriceAlertEnabled(assetId: AssetId): Flow<Boolean> =
        priceAlertStore.observeAssetPriceAlert(assetId)
            .mapLatest { it?.priceAlert != null }
            .flowOn(Dispatchers.IO)
}
