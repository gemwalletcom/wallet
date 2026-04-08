package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.AssetPriceDataAggregate
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.compactFormatter
import com.gemwallet.android.model.format
import com.gemwallet.android.model.shouldUseCompactFormatter
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class GetActiveAssetsInfoImpl(
    private val assetsRepository: AssetsRepository,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetActiveAssetsInfo {
    private val assetsInfo = assetsRepository.getAssetsInfo()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> =
        assetsInfo
            .map { items -> items.map { it.toAssetInfoDataAggregate(hideBalance) } }
            .distinctUntilChanged()
            .flowOn(Dispatchers.Default)
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
    } else if (shouldUseCompactFormatter(assetBalance.totalAmount)) {
        asset.compactFormatter(value = assetBalance.totalAmount)
    } else {
        asset.format(
            humanAmount = assetBalance.totalAmount,
            decimalPlace = 2,
            maxDecimals = 4,
            dynamicPlace = true,
        )
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
