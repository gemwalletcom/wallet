package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.id
import com.gemwallet.android.ext.type
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.CurrencyFormatter
import uniffi.gemstone.PriceAlertFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertNotificationType
import uniffi.gemstone.GemPriceAlertKind
import uniffi.gemstone.GemPriceAlertRow
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetPriceAlertsImpl(
    private val priceAlertStore: GemstonePriceAlertStore,
    private val getWalletAssets: GetWalletAssets,
    private val priceAlertFormatter: PriceAlertFormatter,
) : GetPriceAlerts {
    override fun assetPriceAlerts(assetId: AssetId): Flow<List<PriceAlert>> =
        priceAlertStore.observePriceAlerts(assetId).mapLatest { alerts -> alerts.map { it.priceAlert } }

    override fun invoke(assetId: AssetId?): Flow<List<PriceAlertDataAggregate>> {
        return priceAlertStore.observePriceAlerts(assetId)
            .flatMapLatest { items ->
                val index = priceAlertFormatter.displayedAlertIds(items.map { it.priceAlert.toGem() })
                    .mapNotNull { id -> items.firstOrNull { it.priceAlert.id == id } }
                    .groupBy { it.priceAlert.assetId.toIdentifier() }
                getWalletAssets.byIdentifiers(index.keys.toList()).mapLatest { assetInfos ->
                    assetInfos.flatMap { assetInfo ->
                        index[assetInfo.id().toIdentifier()]?.map { item ->
                            PriceAlertDataAggregateImpl(
                                id = item.id,
                                asset = assetInfo.asset,
                                assetPrice = assetInfo.price,
                                priceAlert = item.priceAlert,
                                row = priceAlertFormatter.row(
                                    alert = item.priceAlert.toGem(),
                                    currentPrice = assetInfo.price?.price?.price,
                                    priceChangePercentage24h = assetInfo.price?.price?.priceChangePercentage24h,
                                    priceCurrency = (assetInfo.price?.currency ?: item.priceAlert.currency).string,
                                ),
                            )
                        }.orEmpty()
                    }
                }
            }
    }
}

@Stable
class PriceAlertDataAggregateImpl(
    override val id: String,
    override val asset: Asset,
    val assetPrice: AssetPriceInfo?,
    override val priceAlert: PriceAlert,
    private val row: GemPriceAlertRow,
) : PriceAlertDataAggregate {
    override val assetId: AssetId = asset.id
    override val title: String = asset.name
    override val titleBadge: String = asset.symbol.uppercase()

    override val priceDirection = row.direction

    override val price: String
        get() = row.price?.let { CurrencyFormatter(currency = row.priceCurrency.toCurrency()).string(it) }.orEmpty()

    override val percentage: String
        get() = row.percent?.let { percent ->
            when (priceAlert.pricePercentChange) {
                null -> percent.formatAsPercentage()
                else -> percent.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)
            }
        }.orEmpty()

    override val kind: GemPriceAlertKind = row.kind

    override val hasTarget: Boolean
        get() = priceAlert.type != PriceAlertNotificationType.Auto

}
