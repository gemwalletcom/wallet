package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.models.CryptoFormattedUIModel
import com.gemwallet.android.ui.models.FiatFormattedUIModel
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PriceUIModel
import com.gemwallet.android.ui.theme.Spacer2
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.wallet.core.primitives.Asset

private val balanceInfoMinWidth = 96.dp
private val balanceInfoMaxWidth = 136.dp

@Composable
fun AssetListItem(
    asset: AssetInfoDataAggregate,
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
) {
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        leading = @Composable { AssetIcon(asset.asset) },
        title = @Composable { ListItemTitleText(asset.title) },
        subtitle = asset.price?.let {
            {
                PriceInfo(
                    it.valueFormatted,
                    it.changePercentageFormatted,
                    it.state,
                )
            }
        },
        trailing = { getBalanceInfo(asset.balance, asset.balanceEquivalent, asset.isZeroBalance).invoke() },
    )
}

@Composable
fun AssetListItem(
    asset: AssetItemUIModel,
    support: @Composable (() -> Unit)?,
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
    badge: String? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        leading = @Composable { AssetIcon(asset.asset) },
        title = @Composable { ListItemTitleText(asset.name, { Badge(text = badge) }) },
        subtitle = support,
        trailing = if (trailing == null) null else {
            { trailing.invoke() }
        }
    )
}

@Composable
fun AssetListItem(
    asset: Asset,
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
    support: String? = null,
    badge: String? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        leading = @Composable { AssetIcon(asset) },
        title = @Composable { ListItemTitleText(asset.name, { Badge(text = badge) }) },
        subtitle = if (support.isNullOrEmpty()) null else {
            { ListItemSupportText(support) }
        },
        trailing = if (trailing == null) null else {
            { trailing.invoke() }
        }
    )
}

@Composable
fun Badge(text: String?) {
    if (text.isNullOrEmpty()) {
        return
    }
    Text(
        modifier = Modifier,
        text = text,
        color = MaterialTheme.colorScheme.secondary,
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.W400,
    )
}

@Composable
fun PriceInfo(
    price: PriceUIModel,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    isHighlightPercentage: Boolean = false,
    internalPadding: Dp = paddingHalfSmall,
) {
    PriceInfo(
        price.fiatFormatted,
        price.percentageFormatted,
        price.state,
        modifier,
        style,
        isHighlightPercentage,
        internalPadding,
    )
}

@Composable
fun PriceInfo(
    price: String,
    changes: String,
    state: PriceState,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    isHighlightPercentage: Boolean = false,
    internalPadding: Dp = paddingHalfSmall,
) {
    val color = state.color()
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(internalPadding)
    ) {
        Text(
            modifier = Modifier.weight(1f, false),
            text = price,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
            color = if (isHighlightPercentage) color else MaterialTheme.colorScheme.secondary,
            style = style,
        )
        Text(
            modifier = if (isHighlightPercentage) {
                Modifier.background(color.copy(alpha = alpha10), MaterialTheme.shapes.small)
            } else {
                Modifier
            },//.padding(horizontal = 4.dp),
            text = changes,
            color = color,
            style = style,
        )
    }
}

fun getBalanceInfo(uiModel: AssetItemUIModel): @Composable () -> Unit
        = getBalanceInfo(uiModel, uiModel)

fun getBalanceInfo(crypto: CryptoFormattedUIModel, fiatFormattedUIModel: FiatFormattedUIModel): @Composable () -> Unit {
    return (@Composable {
        val color = MaterialTheme.colorScheme.let {
            if (crypto.isZeroAmount) it.secondary else it.onSurface
        }
        BalanceInfo(
            crypto = crypto.cryptoFormatted,
            equivalent = fiatFormattedUIModel.fiatFormatted.takeIf { !crypto.isZeroAmount }.orEmpty(),
            color = color,
        )
    })
}

fun getBalanceInfo(crypto: String, equivalent: String, isZero: Boolean): @Composable () -> Unit {
    return (@Composable {
        val color = MaterialTheme.colorScheme.let {
            if (isZero) it.secondary else it.onSurface
        }
        BalanceInfo(
            crypto = crypto,
            equivalent = equivalent.takeIf { !isZero }.orEmpty(),
            color = color,
        )
    })
}

@Composable
private fun BalanceInfo(
    crypto: String,
    equivalent: String,
    color: Color,
) {
    Column(
        modifier = Modifier.widthIn(
            min = balanceInfoMinWidth,
            max = balanceInfoMaxWidth,
        ),
        horizontalAlignment = Alignment.End
    ) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            text = crypto,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
            textAlign = TextAlign.End,
            style = MaterialTheme.typography.titleMedium,
            color = color,
        )
        if (equivalent.isNotEmpty()) {
            Spacer2()
            Text(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 0.dp, bottom = 2.dp),
                text = equivalent,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.End,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
fun PriceInfo(
    priceValue: String,
    changedPercentages: String,
    state: PriceState,
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.secondary,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    isHighlightPercentage: Boolean = false,
    internalPadding: Dp = 16.dp,
) {
    val highlightColor = state.color()
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = priceValue,
            color = if (isHighlightPercentage) highlightColor else color,
            style = style,
        )
        Spacer(modifier = Modifier.width(internalPadding))
        Text(
            modifier = if (isHighlightPercentage) {
                Modifier.background(highlightColor.copy(alpha = alpha10), MaterialTheme.shapes.small)
            } else {
                Modifier
            }.padding(4.dp),
            text = changedPercentages,
            color = highlightColor,
            style = style,
        )
    }
}

//@Composable
//fun priceColor(state: PriceState) = when (state) {
//    PriceState.Up -> MaterialTheme.colorScheme.tertiary
//    PriceState.Down -> MaterialTheme.colorScheme.error
//    PriceState.None -> MaterialTheme.colorScheme.secondary
//}

@Composable
fun PriceState.color() = when (this) {
    PriceState.Up -> MaterialTheme.colorScheme.tertiary
    PriceState.Down -> MaterialTheme.colorScheme.error
    PriceState.None -> MaterialTheme.colorScheme.secondary
}
