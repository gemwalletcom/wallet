package com.gemwallet.android.data.coordinators.pricealerts

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest
import uniffi.gemstone.GemPriceAlertRow
import uniffi.gemstone.GemPriceAlertText
import uniffi.gemstone.PriceAlertFormatter

@OptIn(ExperimentalCoroutinesApi::class)
class GetPriceAlertsImpl(private val priceAlertStore: GemstonePriceAlertStore, private val getWalletAssets: GetWalletAssets, private val priceAlertFormatter: PriceAlertFormatter) : GetPriceAlerts {
    override fun assetPriceAlerts(assetId: AssetId): Flow<List<PriceAlert>> = priceAlertStore.observePriceAlerts(assetId).mapLatest { alerts -> alerts.map { it.priceAlert } }

    override fun invoke(assetId: AssetId?): Flow<List<PriceAlertDataAggregate>> = priceAlertStore.observePriceAlerts(assetId)
        .flatMapLatest { items ->
            val index = priceAlertFormatter.displayedAlertIds(items.map { it.priceAlert.toGem() })
                .mapNotNull { id -> items.firstOrNull { it.id == id } }
                .groupBy { it.priceAlert.assetId.toIdentifier() }
            getWalletAssets.byIdentifiers(index.keys.toList()).mapLatest { assetInfos ->
                assetInfos.flatMap { assetInfo ->
                    index[assetInfo.id().toIdentifier()]?.map { item ->
                        PriceAlertDataAggregateImpl(
                            id = item.id,
                            asset = assetInfo.asset,
                            rankScore = assetInfo.metadata.rankScore,
                            priceAlert = item.priceAlert,
                            row = priceAlertFormatter.row(
                                data = PriceAlertData(
                                    asset = assetInfo.asset,
                                    price = assetInfo.price?.price?.let {
                                        Price(
                                            price = it.price,
                                            priceChangePercentage24h = it.priceChangePercentage24h,
                                            updatedAt = it.updatedAt,
                                        )
                                    },
                                    priceAlert = item.priceAlert,
                                    rankScore = assetInfo.metadata.rankScore,
                                ).toGem(),
                                priceCurrency = (assetInfo.price?.currency ?: item.priceAlert.currency).toGem(),
                            ),
                        )
                    }.orEmpty()
                }
            }
        }
}

@Stable
class PriceAlertDataAggregateImpl(override val id: String, override val asset: Asset, override val rankScore: Int, override val priceAlert: PriceAlert, private val row: GemPriceAlertRow) : PriceAlertDataAggregate {
    override val assetId: AssetId = asset.id
    override val title: String = row.title
    override val titleBadge: String? = row.symbol

    override val priceDirection = row.direction

    override val prefix: GemPriceAlertText = row.prefix

    override val suffix: GemPriceAlertText = row.suffix
}
