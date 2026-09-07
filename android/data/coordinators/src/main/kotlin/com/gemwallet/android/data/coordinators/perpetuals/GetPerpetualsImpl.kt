package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.domains.price.values.RowFormatters
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualsImpl @Inject constructor(
    private val perpetualStore: GemstonePerpetualStore,
) : GetPerpetuals {

    override fun getPerpetuals(searchQuery: String?): Flow<List<PerpetualDataAggregate>> {
        return perpetualStore.observePerpetuals(searchQuery)
            .map { items ->
                val formatters = RowFormatters()
                items.map { PerpetualDataAggregate(it, formatters) }
            }
            .flowOn(Dispatchers.Default)
    }

    class PerpetualDataAggregate(
        val data: PerpetualData,
        formatters: RowFormatters,
    ) : com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate {

        override val price: EquivalentValue = formatters.price(Currency.USD, data.perpetual.price, data.perpetual.pricePercentChange24h)

        override val id: PerpetualId = data.perpetual.id

        override val asset: Asset = data.asset

        override val name: String = data.perpetual.name

        override val volume: String = formatters.abbreviated(price.currency).string(data.perpetual.volume24h)

        override val isPinned: Boolean = data.metadata.isPinned
    }
}
