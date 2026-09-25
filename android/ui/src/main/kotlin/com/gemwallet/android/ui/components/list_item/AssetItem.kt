package com.gemwallet.android.ui.components.list_item

import android.content.Context
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.balance.hiddenWhen
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemValueTone

@Composable
private fun assetListItemContentPadding(): Dp = adaptivePadding(default = paddingMiddle, compact = space6)

sealed interface AssetItemAction {
    data class Switch(val enabled: Boolean) : AssetItemAction
    data object Copy : AssetItemAction
}

@Composable
fun AssetListItem(asset: AssetInfoDataAggregate, listPosition: ListPosition, modifier: Modifier = Modifier, onAction: ((AssetItemAction) -> Unit)? = null) {
    AssetListItem(row = asset.row, listPosition = listPosition, modifier = modifier, hideBalance = asset.hideBalance, onAction = onAction)
}

@Composable
fun AssetListItem(row: GemAssetItemRow, listPosition: ListPosition, modifier: Modifier = Modifier, hideBalance: Boolean = false, onAction: ((AssetItemAction) -> Unit)? = null, accessory: (@Composable () -> Unit)? = null) {
    val context = LocalContext.current
    val hidden = hideBalance && row.masksBalance
    val trailing: (@Composable () -> Unit)? = when (val value = row.trailing) {
        is GemAssetItemTrailing.Value -> getBalanceInfo(value.value, value.extra, hidden)

        is GemAssetItemTrailing.Toggle -> {
            { Switch(checked = value.isOn, onCheckedChange = { onAction?.invoke(AssetItemAction.Switch(it)) }) }
        }

        GemAssetItemTrailing.Copy -> {
            {
                IconButton(onClick = { onAction?.invoke(AssetItemAction.Copy) }, modifier = Modifier.size(iconSize)) {
                    Icon(
                        imageVector = AppIcons.ContentCopyOutlined,
                        contentDescription = "",
                        modifier = Modifier.size(compactIconSize),
                        tint = MaterialTheme.colorScheme.secondary,
                    )
                }
            }
        }

        GemAssetItemTrailing.None -> accessory
    }
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        minHeight = ListItemDefaults.iconMinHeight,
        contentPadding = assetListItemContentPadding(),
        titleSubtitleSpacing = space0,
        leading = @Composable { AssetIcon(row.icon) },
        title = @Composable { ListItemTitleText(row.title, { Badge(text = row.titleExtra) }) },
        subtitle = row.subtitle?.let { subtitle -> { AssetItemSupport(subtitle.string(context), subtitle.tone, row.subtitleExtra?.let { it.string(context) to it.tone }) } },
        trailing = trailing?.let { content -> { content() } },
    )
}

@Composable
private fun AssetItemSupport(text: String, tone: GemValueTone, extra: Pair<String, GemValueTone>?) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        Text(
            modifier = Modifier.weight(1f, false),
            text = text,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
            color = tone.color(),
            style = MaterialTheme.typography.bodyMedium,
        )
        extra?.let { (extraText, extraTone) ->
            Text(
                text = extraText,
                maxLines = 1,
                color = extraTone.color(),
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
fun AssetListItem(
    asset: Asset,
    listPosition: ListPosition,
    modifier: Modifier = Modifier,
    icon: GemAssetIcon? = null,
    title: String = asset.name,
    support: @Composable (() -> Unit)? = null,
    badge: String? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        minHeight = ListItemDefaults.iconMinHeight,
        contentPadding = assetListItemContentPadding(),
        titleSubtitleSpacing = space0,
        leading = @Composable { icon?.let { AssetIcon(it) } ?: AssetIcon(asset) },
        title = @Composable { ListItemTitleText(title, { Badge(text = badge) }) },
        subtitle = support,
        trailing = if (trailing == null) {
            null
        } else {
            { trailing.invoke() }
        },
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

fun getBalanceInfo(crypto: String, equivalent: String, isZero: Boolean): @Composable () -> Unit = (
    @Composable {
        val color = MaterialTheme.colorScheme.let {
            if (isZero) it.secondary else it.onSurface
        }
        BalanceInfo(
            crypto = crypto,
            equivalent = equivalent.takeIf { !isZero }.orEmpty(),
            color = color,
        )
    }
    )

fun getBalanceInfo(value: GemRowText, extra: GemRowText?, hidden: Boolean = false): @Composable () -> Unit = (
    @Composable {
        val context = LocalContext.current
        BalanceInfo(
            crypto = value.string(context).hiddenWhen(hidden),
            equivalent = extra?.string(context)?.hiddenWhen(hidden).orEmpty(),
            color = value.tone.color(),
            equivalentColor = extra?.tone?.color() ?: MaterialTheme.colorScheme.secondary,
        )
    }
    )

fun getBalanceInfo(amount: GemFormattedNumber, equivalent: GemFormattedNumber?): @Composable () -> Unit = (
    @Composable {
        BalanceInfo(crypto = amount.text(), equivalent = equivalent?.text().orEmpty(), color = amount.tone.color())
    }
    )

@Composable
private fun BalanceInfo(crypto: String, equivalent: String, color: Color, equivalentColor: Color = MaterialTheme.colorScheme.secondary) {
    Column(
        horizontalAlignment = Alignment.End,
    ) {
        Text(
            modifier = Modifier,
            text = crypto,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
            textAlign = TextAlign.End,
            style = MaterialTheme.typography.titleMedium,
            color = color,
        )
        if (equivalent.isNotEmpty()) {
            Text(
                modifier = Modifier,
                text = equivalent,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.End,
                color = equivalentColor,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}
