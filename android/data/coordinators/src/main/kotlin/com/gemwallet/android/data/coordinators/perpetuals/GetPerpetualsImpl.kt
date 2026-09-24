package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualSections
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemPerpetualMarketRow
import uniffi.gemstone.GemPriceRow
import uniffi.gemstone.perpetualMarketQuery
import uniffi.gemstone.perpetualMarketRows
import uniffi.gemstone.perpetualMarketSections
import javax.inject.Inject

class GetPerpetualsImpl @Inject constructor(private val perpetualStore: GemstonePerpetualStore) : GetPerpetuals {

    override fun getPerpetuals(searchQuery: String?): Flow<List<PerpetualDataAggregate>> = perpetualStore.observePerpetuals(perpetualMarketQuery(searchQuery.orEmpty()))
        .map { items -> items.zip(perpetualMarketRows(items.map { it.toGem() }), ::PerpetualDataAggregate) }
        .flowOn(Dispatchers.Default)

    override fun getPerpetualSections(searchQuery: Flow<String?>): Flow<PerpetualSections> = searchQuery
        .flatMapLatest { query -> perpetualStore.observePerpetuals(perpetualMarketQuery(query.orEmpty())) }
        .map { items ->
            val sections = perpetualMarketSections(items.map { it.toGem() })
            PerpetualSections(
                pinned = sections.pinned.map { PerpetualDataAggregate(it.data.toPrimitives(), it.row) },
                markets = sections.markets.map { PerpetualDataAggregate(it.data.toPrimitives(), it.row) },
            )
        }
        .flowOn(Dispatchers.Default)

    class PerpetualDataAggregate(val data: PerpetualData, row: GemPerpetualMarketRow) : com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate {

        override val id: PerpetualId = data.perpetual.id

        override val asset: Asset = data.asset

        override val icon: GemAssetIcon = row.icon

        override val price: GemPriceRow = row.price

        override val title: String = row.title

        override val volume: String = row.volume24h.text()

        override val isPinned: Boolean = data.metadata.isPinned
    }
}
