package com.gemwallet.android.features.perpetuals.presents.components

import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemPerpetualMarketItem
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.assetText
import uniffi.gemstone.formattedCurrency
import uniffi.gemstone.formattedPercentage

@Composable
fun PerpetualListItem(item: GemPerpetualMarketItem, modifier: Modifier = Modifier, listPosition: ListPosition = ListPosition.Single, longPressState: MutableState<PerpetualId?>, onTogglePin: (PerpetualId) -> Unit, onClick: () -> Unit) {
    val perpetualId = PerpetualId(item.data.perpetual.id)
    val isPinned = item.data.metadata.isPinned
    DropDownContextItem(
        modifier = modifier,
        isExpanded = longPressState.value == perpetualId,
        onDismiss = { longPressState.value = null },
        content = {
            PerpetualListItem(
                modifier = it,
                item = item,
                listPosition = listPosition,
            )
        },
        menuItems = {
            DropdownMenuItem(
                text = { Text(text = stringResource(id = if (isPinned) R.string.common_unpin else R.string.common_pin)) },
                trailingIcon = {
                    if (isPinned) {
                        Icon(painterResource(R.drawable.keep_off), "unpin")
                    } else {
                        Icon(AppIcons.PushPin, "pin")
                    }
                },
                onClick = {
                    onTogglePin(perpetualId)
                    longPressState.value = null
                },
            )
        },
        onLongClick = { longPressState.value = perpetualId },
    ) { onClick() }
}

@Composable
fun PerpetualListItem(item: GemPerpetualMarketItem, modifier: Modifier = Modifier, listPosition: ListPosition = ListPosition.Single) {
    AssetListItem(
        row = item.row,
        modifier = modifier,
        listPosition = listPosition,
    )
}

@Preview
@Composable
private fun PerpetualListItemPreview() {
    WalletTheme {
        PerpetualListItem(item = previewPerpetual(Asset(id = AssetId(Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE), "BTC", 95420.50, 2.5, "$15.0B"))
    }
}

internal fun previewPerpetual(asset: Asset, title: String, price: Double, change: Double, volume: String, isPinned: Boolean = false) = GemPerpetualMarketItem(
    data = PerpetualData(
        perpetual = Perpetual(
            id = PerpetualId(PerpetualProvider.Hypercore, asset.symbol),
            name = asset.symbol,
            provider = PerpetualProvider.Hypercore,
            assetId = asset.id,
            identifier = asset.symbol,
            price = price,
            pricePercentChange24h = change,
            openInterest = 0.0,
            volume24h = 0.0,
            funding = 0.0,
            maxLeverage = 1u,
            isIsolatedOnly = false,
        ),
        asset = asset,
        metadata = PerpetualMetadata(isPinned),
    ).toGem(),
    row = GemAssetItemRow(
        icon = assetText(asset.toGem()).icon,
        title = title,
        titleExtra = null,
        subtitle = GemRowText(GemLocalizedText.Number(formattedCurrency(price, Currency.USD.string, GemCurrencyStyle.SHORT)), GemValueTone.NEUTRAL),
        subtitleExtra = GemRowText(GemLocalizedText.Number(formattedPercentage(change, GemPercentageStyle.SIGNED)), if (change < 0) GemValueTone.NEGATIVE else GemValueTone.POSITIVE),
        trailing = GemAssetItemTrailing.Value(GemRowText(GemLocalizedText.Text(volume), GemValueTone.PLAIN), null),
        masksBalance = false,
    ),
)
