package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.priceRow

private val trailingMinWidth = 40.dp

@Composable
fun PerpetualItem(item: PerpetualDataAggregate, modifier: Modifier = Modifier, listPosition: ListPosition = ListPosition.Single, longPressState: MutableState<PerpetualId?>, onTogglePin: (PerpetualId) -> Unit, onClick: (AssetId) -> Unit) {
    DropDownContextItem(
        modifier = modifier,
        isExpanded = longPressState.value == item.id,
        onDismiss = { longPressState.value = null },
        content = {
            PerpetualItem(
                modifier = it,
                item = item,
                listPosition = listPosition,
            )
        },
        menuItems = {
            DropdownMenuItem(
                text = { Text(text = stringResource(id = if (item.isPinned) R.string.common_unpin else R.string.common_pin)) },
                trailingIcon = {
                    if (item.isPinned) {
                        Icon(painterResource(R.drawable.keep_off), "unpin")
                    } else {
                        Icon(AppIcons.PushPin, "pin")
                    }
                },
                onClick = {
                    onTogglePin(item.id)
                    longPressState.value = null
                },
            )
        },
        onLongClick = { longPressState.value = item.id },
    ) { onClick(item.asset.id) }
}

@Composable
fun PerpetualItem(item: PerpetualDataAggregate, modifier: Modifier = Modifier, listPosition: ListPosition = ListPosition.Single) {
    AssetListItem(
        asset = item.asset,
        icon = item.icon,
        title = item.title,
        modifier = modifier,
        listPosition = listPosition,
        support = item.price.price?.let { price ->
            {
                PriceInfo(
                    price = price.text(),
                    changes = item.price.change?.text().orEmpty(),
                    changeStyle = (item.price.change?.tone ?: GemValueTone.PLAIN).textStyle(),
                    style = MaterialTheme.typography.bodyMedium,
                    internalPadding = paddingHalfSmall,
                )
            }
        },
        trailing = {
            Column(
                modifier = Modifier.defaultMinSize(trailingMinWidth),
                horizontalAlignment = Alignment.End,
            ) {
                ListItemTitleText(item.volume, color = MaterialTheme.colorScheme.onSurface)
            }
        },
    )
}

@Preview
@Composable
private fun PerpetualItemPreview() {
    val sampleData = object : PerpetualDataAggregate {
        override val id = PerpetualId(PerpetualProvider.Hypercore, "BTC")
        override val title = "BTC"
        override val asset = Asset(
            id = AssetId(Chain.Bitcoin),
            name = "Bitcoin",
            symbol = "BTC",
            decimals = 8,
            type = AssetType.NATIVE,
        )
        override val isPinned: Boolean = true
        override val price = priceRow(price = 95420.50, change = 2.5, currency = Currency.USD.toGem(), style = GemCurrencyStyle.SHORT)
        override val volume = "$15.0B"
    }

    WalletTheme {
        PerpetualItem(item = sampleData)
    }
}
