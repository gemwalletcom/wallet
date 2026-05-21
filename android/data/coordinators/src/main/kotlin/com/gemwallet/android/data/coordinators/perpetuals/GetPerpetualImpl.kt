package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualImpl @Inject constructor(
    private val perpetualRepository: PerpetualRepository
) : GetPerpetual {

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualDetailsDataAggregate?> {
        return perpetualRepository.getPerpetual(perpetualId).map {
            PerpetualDetailsDataAggregateImpl(it ?: return@map null)
        }
    }

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualDetailsDataAggregate?> {
        return perpetualRepository.getPerpetualByAssetId(assetId).map {
            PerpetualDetailsDataAggregateImpl(it ?: return@map null)
        }
    }
}

class PerpetualDetailsDataAggregateImpl(
    private val data: PerpetualData,
) : PerpetualDetailsDataAggregate {
    private val abbreviatedFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD)

    override val id: PerpetualId = data.perpetual.id

    override val provider: PerpetualProvider = data.perpetual.provider

    override val asset: Asset = data.asset

    override val name: String = data.perpetual.name

    override val dayVolume: String = abbreviatedFormatter.string(data.perpetual.volume24h)

    override val openInterest: String = abbreviatedFormatter.string(data.perpetual.openInterest)

    override val funding: String = (data.perpetual.funding * HOURS_PER_YEAR).formatAsPercentage()

    override val maxLeverage: Int = data.perpetual.maxLeverage.toInt()

    override val price: Double = data.perpetual.price

    override val identifier: String = data.perpetual.identifier

    override val isIsolatedOnly: Boolean = data.perpetual.isIsolatedOnly

    private companion object {
        const val HOURS_PER_YEAR = 24 * 365
    }
}
