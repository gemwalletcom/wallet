package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class GetActiveAssetsInfoImpl(
    private val assetsRepository: AssetsRepository,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetActiveAssetsInfo {
    private val assetsInfo = assetsRepository.getAssetsInfo()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> {
        return assetsInfo.map { items -> items.map { AssetInfoDataAggregateImpl(it, hideBalance) } }
    }
}

@Stable
class AssetInfoDataAggregateImpl(
    private val assetInfo: AssetInfo,
    private val hideBalance: Boolean,
) : AssetInfoDataAggregate {
    override val id: AssetId = assetInfo.asset.id

    override val asset: Asset = assetInfo.asset

    override val title: String = assetInfo.asset.name

    override val balance: String
        get() = if (hideBalance) "*****" else assetInfo.asset.format(
            humanAmount = assetInfo.balance.totalAmount,
            decimalPlace = 2,
            maxDecimals = 4,
            dynamicPlace = true
        )

    override val balanceEquivalent: String
        get() {
            val price = assetInfo.price?.price?.price ?: 0.0
            val fiat = if (price == 0.0) null else assetInfo.balance.totalAmount * price
            return if (hideBalance) "*****" else fiat?.let { currency.format(it, dynamicPlace = true) } ?: ""
        }

    override val isZeroBalance: Boolean
        get() = assetInfo.balance.totalAmount == 0.0

    override val price: EquivalentValue?
        get() = assetInfo.price?.price?.let {
            EquivalentValueImpl(
                currency = currency,
                value = it.price,
                changePercentage = it.priceChangePercentage24h,
            )
        }

    override val position: Int
        get() = assetInfo.position

    override val pinned: Boolean
        get() = assetInfo.metadata?.isPinned == true

    override val accountAddress: String
        get() = assetInfo.owner?.address ?: ""

    private val currency: Currency = assetInfo.price?.currency ?: Currency.USD

    private class EquivalentValueImpl(
        override val currency: Currency,
        override val value: Double?,
        override val changePercentage: Double?
    ) : EquivalentValue
}
