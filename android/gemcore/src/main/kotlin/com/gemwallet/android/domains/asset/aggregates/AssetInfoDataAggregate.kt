package com.gemwallet.android.domains.asset.aggregates

import androidx.compose.runtime.Immutable
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency

@Immutable
data class AssetInfoDataAggregate(
    val id: AssetId,
    val asset: Asset,
    val title: String,
    val balance: String,
    val balanceEquivalent: String,
    val isZeroBalance: Boolean,
    val price: AssetPriceDataAggregate?,
    val position: Int,
    val pinned: Boolean,
    val accountAddress: String,
)

@Immutable
data class AssetPriceDataAggregate(
    override val currency: Currency,
    override val value: Double?,
    override val changePercentage: Double?,
) : EquivalentValue
