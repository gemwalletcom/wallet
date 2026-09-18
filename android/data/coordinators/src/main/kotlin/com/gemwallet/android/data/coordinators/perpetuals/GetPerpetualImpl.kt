package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.perpetualMarketRow
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualImpl @Inject constructor(
    private val perpetualStore: GemstonePerpetualStore
) : GetPerpetual {

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualDetailsDataAggregate?> {
        return perpetualStore.observePerpetual(perpetualId).map {
            PerpetualDetailsDataAggregateImpl(it ?: return@map null)
        }
    }

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualDetailsDataAggregate?> {
        return perpetualStore.observePerpetualByAssetId(assetId).map {
            PerpetualDetailsDataAggregateImpl(it ?: return@map null)
        }
    }
}

class PerpetualDetailsDataAggregateImpl(
    private val data: PerpetualData,
) : PerpetualDetailsDataAggregate {
    private val row = perpetualMarketRow(data.perpetual.toGem(), data.asset.toGem())

    override val perpetual: Perpetual = data.perpetual

    override val id: PerpetualId = data.perpetual.id

    override val provider: PerpetualProvider = data.perpetual.provider

    override val asset: Asset = data.asset

    override val name: String = row.title

    override val maxLeverage: Int = data.perpetual.maxLeverage.toInt()

    override val price: Double = data.perpetual.price

    override val identifier: String = data.perpetual.identifier

    override val isIsolatedOnly: Boolean = data.perpetual.isIsolatedOnly
}
