package com.gemwallet.android.domains.asset.aggregates

import androidx.compose.runtime.Immutable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemAssetListRowInput
import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemRowText
import uniffi.gemstone.assetListRow
import uniffi.gemstone.assetListRows
import uniffi.gemstone.walletAssetRows

@Immutable
data class AssetInfoDataAggregate(val asset: Asset, val row: GemAssetItemRow, val hideBalance: Boolean, val pinned: Boolean, val balanceEnabled: Boolean, val accountAddress: String) {
    val id: AssetId get() = asset.id
}

val GemAssetItemRow.trailingValue: GemRowText?
    get() = (trailing as? GemAssetItemTrailing.Value)?.value

fun List<AssetInfo>.toAssetInfoDataAggregates(style: GemAssetRowStyle? = null, hideBalance: Boolean = false): List<AssetInfoDataAggregate> {
    val inputs = map { it.rowInput(GemAssetBalanceScope.TOTAL) }
    val rows = style?.let { assetListRows(inputs, it) } ?: walletAssetRows(inputs)
    return zip(rows) { info, row -> info.aggregate(row, hideBalance) }
}

fun AssetInfo.toAssetInfoDataAggregate(style: GemAssetRowStyle, hideBalance: Boolean = false, scope: GemAssetBalanceScope = GemAssetBalanceScope.TOTAL): AssetInfoDataAggregate = aggregate(assetListRow(rowInput(scope), style), hideBalance)

private fun AssetInfo.rowInput(scope: GemAssetBalanceScope): GemAssetListRowInput {
    val assetPrice = price?.price
    return GemAssetListRowInput(
        asset = asset.toGem(),
        balance = balance.toGem(),
        scope = scope,
        price = assetPrice?.price?.takeIf(Double::isFinite),
        change = assetPrice?.priceChangePercentage24h?.takeIf(Double::isFinite),
        currency = (price?.currency ?: Currency.USD).toGem(),
        isEnabled = metadata.isBalanceEnabled,
    )
}

private fun AssetInfo.aggregate(row: GemAssetItemRow, hideBalance: Boolean): AssetInfoDataAggregate = AssetInfoDataAggregate(
    asset = asset,
    row = row,
    hideBalance = hideBalance,
    pinned = metadata.isPinned,
    balanceEnabled = metadata.isBalanceEnabled,
    accountAddress = owner?.address.orEmpty(),
)
