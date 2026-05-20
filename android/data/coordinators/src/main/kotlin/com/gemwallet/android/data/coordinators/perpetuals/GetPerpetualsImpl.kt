package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetuals
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualsImpl @Inject constructor(
    private val perpetualRepository: PerpetualRepository,
) : GetPerpetuals {

    override fun getPerpetuals(searchQuery: String?): Flow<List<PerpetualDataAggregate>> {
        return perpetualRepository.getPerpetuals(searchQuery)
            .map { items -> items.map { PerpetualDataAggregate(it) } }
    }

    class PerpetualDataAggregate(
        val data: PerpetualData,
        override val price: EquivalentValue = object : EquivalentValue { // TODO: ???
            override val value: Double = data.perpetual.price
            override val currency: Currency = Currency.USD
            override val changePercentage: Double = data.perpetual.pricePercentChange24h
        }
    ) : com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate {

        override val id: PerpetualId = data.perpetual.id

        override val asset: Asset = data.asset

        override val name: String = data.perpetual.name

        override val volume: String = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = price.currency).string(data.perpetual.volume24h)

        override val isPinned: Boolean = data.metadata.isPinned
    }
}
