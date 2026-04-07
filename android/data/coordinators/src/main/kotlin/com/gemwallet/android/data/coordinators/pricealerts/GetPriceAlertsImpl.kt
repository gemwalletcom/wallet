package com.gemwallet.android.data.coordinators.pricealerts

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertType
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.shouldDisplay
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetPriceAlertsImpl(
    private val priceAlertRepository: PriceAlertRepository,
    private val assetsRepository: AssetsRepository,
) : GetPriceAlerts {
    override fun invoke(assetId: AssetId?): Flow<List<PriceAlertDataAggregate>> {
        return priceAlertRepository.getPriceAlerts(assetId)
            .flatMapLatest { items ->
                val index = items
                    .filter { it.priceAlert.shouldDisplay }
                    .groupBy { it.priceAlert.assetId.toIdentifier() }
                assetsRepository.getTokensInfo(index.keys.toList()).mapLatest { assetInfos ->
                    assetInfos.flatMap { assetInfo ->
                        index[assetInfo.id().toIdentifier()]?.map { item ->
                            PriceAlertDataAggregateImpl(
                                id = item.id,
                                asset = assetInfo.asset,
                                assetPrice = assetInfo.price,
                                priceAlert = item.priceAlert,
                            )
                        }.orEmpty()
                    }
                }
            }
    }
}

@Stable
class PriceAlertDataAggregateImpl(
    override val id: Int,
    override val asset: Asset,
    val assetPrice: AssetPriceInfo?,
    val priceAlert: PriceAlert
) : PriceAlertDataAggregate {
    override val assetId: AssetId = asset.id
    override val title: String = asset.name
    override val titleBadge: String = asset.symbol.uppercase()

    override val priceState: PriceState get() {
        val alertPrice = priceAlert.price

        return when (priceAlert.priceDirection) {
            PriceAlertDirection.Up -> PriceState.Up
            PriceAlertDirection.Down -> PriceState.Down
            else -> if (alertPrice != null) {
                assetPrice?.price?.price?.let { currentPrice ->
                    when {
                        alertPrice > currentPrice -> PriceState.Up
                        else -> PriceState.Down
                    }
                } ?: PriceState.None
            } else {
                assetPrice?.price?.priceChangePercentage24h.toPriceState()
            }
        }
    }

    override val price: String
        get() = priceAlert.price?.let {
            Currency.entries.firstOrNull { it.string == priceAlert.currency }?.format(it) ?: ""
        } ?: assetPrice?.let { it.currency.format(it.price.price) }.orEmpty()

    override val percentage: String
        get() = priceAlert.pricePercentChange?.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)
            ?: assetPrice?.price?.priceChangePercentage24h?.formatAsPercentage().orEmpty()

    override val type: PriceAlertType get() {
        val alertPrice = priceAlert.price
        val percentage = priceAlert.pricePercentChange

        return when {
            percentage != null -> when (priceAlert.priceDirection) {
                PriceAlertDirection.Up -> PriceAlertType.Increase
                PriceAlertDirection.Down -> PriceAlertType.Decrease
                null -> PriceAlertType.Auto
            }
            alertPrice != null -> when (priceAlert.priceDirection) {
                PriceAlertDirection.Up -> PriceAlertType.Over
                PriceAlertDirection.Down -> PriceAlertType.Under
                null -> PriceAlertType.Auto
            }
            else -> PriceAlertType.Auto
        }
    }
    override val hasTarget: Boolean
        get() = priceAlert.price != null || priceAlert.priceDirection != null

}
