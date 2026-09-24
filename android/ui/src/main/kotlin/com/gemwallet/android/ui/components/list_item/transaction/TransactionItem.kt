package com.gemwallet.android.ui.components.list_item.transaction

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.image.BadgeCircle
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator10
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import uniffi.gemstone.GemTransactionBadge
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemValueTone

private val badgeStartPadding = 5.dp

@Composable
fun TransactionItem(data: TransactionDataAggregate, listPosition: ListPosition, onClick: () -> Unit) {
    val context = LocalContext.current
    val row = remember(data) { data.uiModel(context) }
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        minHeight = ListItemDefaults.iconMinHeight,
        titleSubtitleSpacing = space0,
        leading = { TransactionIcon(data) },
        title = {
            ListItemTitleText(
                text = row.title,
                titleBadge = { TransactionStatusBadge(row) },
            )
        },
        subtitle = row.subtitle?.let { { ListItemSupportText(it) } },
        listPosition = listPosition,
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(
                    text = row.value,
                    color = row.valueTone.color(),
                )
                row.equivalentValue?.let {
                    ListItemSupportText(it)
                }
            }
        },
    )
}

@Composable
private fun TransactionIcon(data: TransactionDataAggregate) = when (data.badge) {
    GemTransactionBadge.INCOMING,
    GemTransactionBadge.OUTGOING,
    -> DirectionBadgedIcon(data)

    GemTransactionBadge.ASSET -> AssetIcon(data.icon)
}

private const val BADGE_ICON_SCALE = 0.65f

@Composable
private fun DirectionBadgedIcon(data: TransactionDataAggregate) {
    val size = listItemIconSize
    val icon = when (data.badge) {
        GemTransactionBadge.INCOMING -> AppIcons.ArrowDownward
        GemTransactionBadge.OUTGOING, GemTransactionBadge.ASSET -> AppIcons.ArrowUpward
    }
    val color = when (data.badge) {
        GemTransactionBadge.INCOMING -> MaterialTheme.colorScheme.tertiary
        GemTransactionBadge.OUTGOING, GemTransactionBadge.ASSET -> MaterialTheme.colorScheme.primary
    }
    IconWithBadge(
        icon = data.nftImageUrl ?: data.icon.iconModel(),
        placeholder = if (data.nftImageUrl != null) "NFT" else data.icon.placeholder,
        size = size,
    ) {
        BadgeCircle(size = size, color = color) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.fillMaxSize(BADGE_ICON_SCALE),
            )
        }
    }
}

@Composable
private fun TransactionStatusBadge(row: TransactionRowUIModel) {
    val text = row.badgeText ?: return
    val color = row.badgeTone.color()
    Row(
        Modifier
            .padding(start = badgeStartPadding)
            .background(
                color = color.copy(alpha = alpha10),
                shape = RoundedCornerShape(space6),
            ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            modifier = Modifier.padding(
                start = badgeStartPadding,
                top = space2,
                end = paddingHalfSmall,
                bottom = space2,
            ),
            text = text,
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.labelMedium,
        )
        if (row.showsProgress) {
            CircularProgressIndicator10(color = color)
            Spacer8()
        }
    }
}

@Composable
@Preview
fun PreviewTransactionItem() {
    MaterialTheme {
        TransactionItem(
            data = object : TransactionDataAggregate {
                override val id = TransactionId(Chain.Bitcoin, "preview-1")
                override val asset = Asset(
                    id = AssetId(Chain.Bitcoin),
                    name = "Bitcoin",
                    symbol = "BTC",
                    decimals = 8,
                    type = AssetType.NATIVE,
                )
                override val icon = asset.id.icon()
                override val value = "-0.9998888999 BTC"
                override val equivalentValue: String? = null
                override val title = GemTransactionTitle.Transfer
                override val status = GemTransactionStatus(tone = GemTransactionStateTone.PENDING, showsBadge = true, showsProgress = true)
                override val subtitle = GemTransactionRowSubtitle.ToAddress("btc12312sdfksdjfks")
                override val valueTone = GemValueTone.PLAIN
                override val badge = GemTransactionBadge.OUTGOING
                override val state = TransactionState.Pending
                override val createdAt = System.currentTimeMillis()
            },
            listPosition = ListPosition.Single,
            onClick = {},
        )
    }
}

@Composable
@Preview
fun PreviewSwapTransactionItem() {
    MaterialTheme {
        TransactionItem(
            data = object : TransactionDataAggregate {
                override val id = TransactionId(Chain.SmartChain, "preview-2")
                override val asset = Asset(
                    id = AssetId(Chain.SmartChain),
                    name = "SmartChain",
                    symbol = "BNB",
                    decimals = 18,
                    type = AssetType.NATIVE,
                )
                override val icon = asset.id.icon()
                override val value = "+19 TON"
                override val equivalentValue = "-0.09 BNB"
                override val status = GemTransactionStatus(tone = GemTransactionStateTone.SUCCESS, showsBadge = false, showsProgress = false)
                override val title = GemTransactionTitle.Swap
                override val subtitle = GemTransactionRowSubtitle.None
                override val valueTone = GemValueTone.POSITIVE
                override val badge = GemTransactionBadge.ASSET
                override val state = TransactionState.Confirmed
                override val createdAt = System.currentTimeMillis()
            },
            listPosition = ListPosition.Single,
            onClick = {},
        )
    }
}
