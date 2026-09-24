package com.gemwallet.android.domains.asset.aggregates

import androidx.compose.runtime.Immutable
import com.gemwallet.android.domains.balance.HIDDEN_BALANCE
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemAssetListRow
import uniffi.gemstone.GemAssetListRowInput
import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemPriceRow
import uniffi.gemstone.assetListRow
import uniffi.gemstone.assetListRows

@Immutable
data class AssetInfoDataAggregate(
    val id: AssetId,
    val asset: Asset,
    val title: String,
    val symbol: String?,
    val network: String?,
    val balance: String,
    val balanceEquivalent: String,
    val isZeroBalance: Boolean,
    val price: GemPriceRow,
    val pinned: Boolean,
    val balanceEnabled: Boolean,
    val accountAddress: String,
)

fun List<AssetInfo>.toAssetInfoDataAggregates(style: GemAssetRowStyle, hideBalance: Boolean = false): List<AssetInfoDataAggregate> =
    zip(assetListRows(map { it.rowInput(style, GemAssetBalanceScope.TOTAL) })) { info, row -> info.aggregate(row, hideBalance) }

fun AssetInfo.toAssetInfoDataAggregate(style: GemAssetRowStyle, hideBalance: Boolean = false, scope: GemAssetBalanceScope = GemAssetBalanceScope.TOTAL): AssetInfoDataAggregate = aggregate(assetListRow(rowInput(style, scope)), hideBalance)

private fun AssetInfo.rowInput(style: GemAssetRowStyle, scope: GemAssetBalanceScope): GemAssetListRowInput {
    val assetPrice = price?.price
    return GemAssetListRowInput(
        asset = asset.toGem(),
        balance = balance.toGem(),
        scope = scope,
        price = assetPrice?.price?.takeIf(Double::isFinite),
        change = assetPrice?.priceChangePercentage24h?.takeIf(Double::isFinite),
        currency = (price?.currency ?: Currency.USD).toGem(),
        style = style,
    )
}

private fun AssetInfo.aggregate(row: GemAssetListRow, hideBalance: Boolean): AssetInfoDataAggregate = AssetInfoDataAggregate(
    id = asset.id,
    asset = asset,
    title = row.text.title,
    symbol = row.text.symbol,
    network = row.text.network,
    balance = if (hideBalance) HIDDEN_BALANCE else row.amount.text(),
    balanceEquivalent = if (hideBalance) HIDDEN_BALANCE else row.fiat?.text().orEmpty(),
    isZeroBalance = !row.hasBalance,
    price = row.price,
    pinned = metadata.isPinned,
    balanceEnabled = metadata.isBalanceEnabled,
    accountAddress = owner?.address.orEmpty(),
)
