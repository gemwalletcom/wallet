package com.gemwallet.android.domains.asset.aggregates

import androidx.compose.runtime.Immutable
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
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

fun List<AssetData>.toAssetInfoDataAggregates(currency: Currency, style: GemAssetRowStyle? = null, hideBalance: Boolean = false): List<AssetInfoDataAggregate> {
    val assets = map { it.toGem() }
    val rows = style?.let { assetListRows(assets, currency.toGem(), it) } ?: walletAssetRows(assets, currency.toGem())
    return zip(rows) { info, row -> info.aggregate(row, hideBalance) }
}

fun AssetData.toAssetInfoDataAggregate(currency: Currency, style: GemAssetRowStyle, hideBalance: Boolean = false, scope: GemAssetBalanceScope = GemAssetBalanceScope.TOTAL): AssetInfoDataAggregate =
    aggregate(assetListRow(toGem(), currency.toGem(), scope, style), hideBalance)

private fun AssetData.aggregate(row: GemAssetItemRow, hideBalance: Boolean): AssetInfoDataAggregate = AssetInfoDataAggregate(
    asset = asset,
    row = row,
    hideBalance = hideBalance,
    pinned = metadata.isPinned,
    balanceEnabled = metadata.isBalanceEnabled,
    accountAddress = account.address,
)
