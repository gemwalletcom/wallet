package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.AssetPriceDataAggregate
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import java.math.BigDecimal

class GetActiveAssetsInfoImpl(
    private val assetsRepository: AssetsRepository,
) : GetActiveAssetsInfo {
    override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> =
        assetsRepository.getAssetsInfo()
            .map { items -> items.map { it.toAssetInfoDataAggregate(hideBalance) } }
            .distinctUntilChanged()
}

internal fun AssetInfo.toAssetInfoDataAggregate(
    hideBalance: Boolean,
): AssetInfoDataAggregate {
    val currency = price?.currency ?: Currency.USD
    val assetPrice = price?.price
    val priceValue = assetPrice?.price?.takeIf(Double::isFinite)
    val changePercentage = assetPrice?.priceChangePercentage24h?.takeIf(Double::isFinite)
    val assetBalance = balance
    val formattedBalance = if (hideBalance) {
        "*****"
    } else {
        ValueFormatter(style = ValueFormatter.Style.Short)
            .string(BigDecimal.valueOf(assetBalance.totalAmount), asset.symbol)
    }
    val balanceEquivalent = if (hideBalance) {
        "*****"
    } else {
        priceValue
            ?.takeUnless { it == 0.0 }
            ?.let { currency.format(assetBalance.totalAmount * it, dynamicPlace = true) }
            .orEmpty()
    }
    val price = assetPrice?.let {
        AssetPriceDataAggregate(
            currency = currency,
            value = priceValue,
            changePercentage = changePercentage,
        )
    }

    return AssetInfoDataAggregate(
        id = asset.id,
        asset = asset,
        title = asset.name,
        balance = formattedBalance,
        balanceEquivalent = balanceEquivalent,
        isZeroBalance = assetBalance.totalAmount == 0.0,
        price = price,
        position = position,
        pinned = metadata?.isPinned == true,
        accountAddress = owner?.address.orEmpty(),
    )
}
