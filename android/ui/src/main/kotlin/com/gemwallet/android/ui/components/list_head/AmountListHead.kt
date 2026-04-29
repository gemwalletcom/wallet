package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.foundation.text.modifiers.TextAutoSizeLayoutScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.AttachMoney
import androidx.compose.material.icons.filled.Autorenew
import androidx.compose.material.icons.filled.QrCode2
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.adaptive.ExperimentalMaterial3AdaptiveApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DisplayText
import com.gemwallet.android.ui.components.HideToggle
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.isHidden
import com.gemwallet.android.ui.components.mask
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.actionIconSize
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space10
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.WalletType
import kotlin.math.floor

private val headerChangeTextHeight = 24.dp

@Composable
fun AmountListHead(
    amount: String,
    hideToggle: HideToggle? = null,
    equivalent: String? = null,
    icon: Any? = null,
    changedValue: String? = null,
    changedPercentages: String? = null,
    changeState: PriceState = PriceState.None,
    actions: (@Composable () -> Unit)? = null,
) {
    val hidden = hideToggle.isHidden
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = paddingDefault, end = paddingDefault, bottom = paddingSmall),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            (icon as? Asset)?.let {
                HeaderIcon(it)
            } ?: IconWithBadge(
                icon = icon,
                size = headerIconSize,
                badgeBackgroundColor = MaterialTheme.colorScheme.surface,
            )

            icon?.let { Spacer16() }

            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
            ) {
                DisplayText(
                    text = amount,
                    hideToggle = hideToggle,
                )
                if (!equivalent.isNullOrEmpty()) {
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        text = equivalent,
                        textAlign = TextAlign.Center,
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.W400,
                    )
                }
                changedValue?.let { value ->
                    val highlightColor = changeState.color()
                    val changeTextStyle = MaterialTheme.typography.titleMedium.copy(
                        fontSize = 17.sp,
                        fontWeight = FontWeight.Medium,
                    )
                    Row(
                        modifier = Modifier.height(headerChangeTextHeight),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(space2),
                    ) {
                        Text(
                            text = hideToggle.mask(value),
                            color = highlightColor,
                            style = changeTextStyle,
                        )
                        if (!hidden && !changedPercentages.isNullOrBlank()) {
                            Text(
                                text = "($changedPercentages)",
                                color = highlightColor,
                                style = changeTextStyle,
                            )
                        }
                    }
                }
            }
            if (actions != null) {
                Spacer(modifier = Modifier.size(space10))
                Box(
                    modifier = Modifier.width(IntrinsicSize.Min)
                ) {
                    actions()
                }
            }
        }
    }
}

@Composable
fun HeaderIcon(
    asset: Asset?,
    iconSize: Dp = headerIconSize,
) {
    if (asset == null) { return }
    AssetIcon(
        asset = asset,
        size = iconSize,
        badgeBackgroundColor = MaterialTheme.colorScheme.surface,
    )
}

@OptIn(ExperimentalMaterial3AdaptiveApi::class)
@Composable
fun AssetHeadActions(
    walletType: WalletType,
    transferEnabled: Boolean,
    operationsEnabled: Boolean,
    onTransfer: (() -> Unit)?,
    onReceive: (() -> Unit)?,
    onBuy: (() -> Unit)?,
    onSwap: (() -> Unit)?,
) {
    var actionFontSize by remember { mutableStateOf(16.sp) }
    if (walletType == WalletType.View) {
        AssetWatchOnly()
        return
    }
    Row(
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onTransfer != null) {
            AmountHeadAction(
                modifier = Modifier.weight(1f),
                title = stringResource(id = R.string.wallet_send),
                imageVector = Icons.AutoMirrored.Default.Send,
                enabled = transferEnabled && operationsEnabled,
                contentDescription = stringResource(id = R.string.wallet_send),
                fontSize = actionFontSize,
                onNextFontSize = {
                    if (actionFontSize > it) actionFontSize = it
                },
                onClick = onTransfer,
            )
        }
        if (onReceive != null) {
            AmountHeadAction(
                modifier = Modifier.weight(1f),
                title = stringResource(id = R.string.wallet_receive),
                imageVector = Icons.Default.QrCode2,
                enabled = operationsEnabled,
                contentDescription = stringResource(id = R.string.wallet_receive),
                fontSize = actionFontSize,
                onNextFontSize = {
                    if (actionFontSize > it) actionFontSize = it
                },
                onClick = onReceive,
            )
        }
        if (onBuy != null) {
            AmountHeadAction(
                modifier = Modifier.weight(1f)
                .testTag("assetBuy"),
                title = stringResource(id = R.string.wallet_buy),
                imageVector = Icons.Default.AttachMoney,
                enabled = operationsEnabled,
                contentDescription = stringResource(id = R.string.wallet_buy),
                fontSize = actionFontSize,
                onNextFontSize = {
                    if (actionFontSize > it) actionFontSize = it
                },
                onClick = onBuy,
            )
        }
        if (onSwap != null) {
            AmountHeadAction(
                modifier = Modifier.weight(1f),
                title = stringResource(id = R.string.wallet_swap),
                imageVector = Icons.Default.Autorenew,
                enabled = operationsEnabled,
                contentDescription = stringResource(id = R.string.wallet_swap),
                fontSize = actionFontSize,
                onNextFontSize = {
                    if (actionFontSize > it) actionFontSize = it
                },
                onClick = onSwap,
            )
        }
    }
}

@Composable
private fun AssetWatchOnly() {
    var showInfoSheet by remember { mutableStateOf<InfoSheetEntity?>(null) }
    val openWatchWalletInfo = { showInfoSheet = InfoSheetEntity.WatchWalletInfo }
    Button(
        modifier = Modifier
            .widthIn(min = SceneSizing.contentMaxWidth)
            .testTag("watchWalletBanner"),
        onClick = openWatchWalletInfo,
        enabled = true,
        colors = ButtonDefaults
            .buttonColors(
                contentColor = MaterialTheme.colorScheme.onSurface,
                containerColor = MaterialTheme.colorScheme.background,
            ),
        contentPadding = PaddingValues(
            horizontal = paddingDefault,
            vertical = paddingMiddle,
        ),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Default.Visibility,
                contentDescription = null,
            )
            Spacer8()
            Text(
                text = stringResource(id = R.string.wallet_watch_tooltip_title),
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.W400,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer8()
            IconButton(
                modifier = Modifier
                    .size(smallIconSize)
                    .testTag("watchWalletInfo"),
                onClick = openWatchWalletInfo,
            ) {
                Icon(
                    imageVector = Icons.Outlined.Info,
                    contentDescription = stringResource(R.string.common_learn_more),
                )
            }
        }
    }
    InfoBottomSheet(item = showInfoSheet, onClose = { showInfoSheet = null })
}

@OptIn(ExperimentalMaterial3AdaptiveApi::class)
@Composable
fun AmountHeadAction(
    title: String,
    fontSize: TextUnit,
    imageVector: ImageVector,
    contentDescription: String,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onNextFontSize: (TextUnit) -> Unit,
    onClick: () -> Unit
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(paddingDefault))
            .clickable(onClick = onClick, enabled = enabled)
            .padding(paddingSmall)
        ,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(paddingSmall / (fontSize.value * 0.5f))
    ) {
        Icon(
            modifier = Modifier
                .requiredSize(actionIconSize)
                .background(
                    color = MaterialTheme.colorScheme.primary.copy(
                        alpha = if (enabled) 1f else alpha50,
                    ),
                    shape = CircleShape
                )
                .padding(paddingDefault)
            ,
            imageVector = imageVector,
            tint = MaterialTheme.colorScheme.onPrimary.copy(
                alpha = if (enabled) 1f else alpha50,
            ),
            contentDescription = contentDescription,
        )
        Text(
            text = title,
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.titleMedium.copy(
                fontWeight = FontWeight.W400,
            ),
            autoSize = ActionTextAutoSize(
                minFontSize = 8.sp,
                maxFontSize = 16.sp,
                layoutSize = fontSize,
                stepSize = 0.25.sp,
                onNewLayoutSize = onNextFontSize,
            ),
            fontSize = fontSize,
            maxLines = 1,
        )
    }
}

private class ActionTextAutoSize(
    private var minFontSize: TextUnit,
    private val maxFontSize: TextUnit,
    private val layoutSize: TextUnit,
    private val stepSize: TextUnit,
    private val onNewLayoutSize: (TextUnit) -> Unit,
) : TextAutoSize {


    override fun TextAutoSizeLayoutScope.getFontSize(
        constraints: Constraints,
        text: AnnotatedString
    ): TextUnit {
        val stepSize = stepSize.toPx()
        val smallest = minFontSize.toPx()
        val largest = maxFontSize.toPx()
        var min = smallest
        var max = largest

        if (layoutSize.toPx() > 0) {
            val layoutResult = performLayout(constraints, text, layoutSize)
            if (!layoutResult.didOverflow()) {
                return layoutSize
            }
        }
        var current = ((min + max) / 2)

        while ((max - min) >= stepSize) {
            val layoutResult = performLayout(constraints, text, current.toSp())
            if (layoutResult.didOverflow()) {
                max = current
            } else {
                min = current
            }
            current = (min + max) / 2
        }
        current = (floor((min - smallest) / stepSize) * stepSize + smallest)

        if ((current + stepSize) <= largest) {
            val layoutResult = performLayout(constraints, text, (current + stepSize).toSp())
            if (!layoutResult.didOverflow()) {
                current += stepSize
            }
        }
        onNewLayoutSize(current.toSp())
        return current.toSp()
    }

    private fun TextLayoutResult.didOverflow() =
        when (layoutInput.overflow) {
            TextOverflow.Clip,
            TextOverflow.Visible -> didOverflowBounds()
            TextOverflow.StartEllipsis,
            TextOverflow.MiddleEllipsis,
            TextOverflow.Ellipsis -> didOverflowByEllipsize()
            else ->
                throw IllegalArgumentException(
                    "TextOverflow type ${layoutInput.overflow} is not supported."
                )
        }

    private fun TextLayoutResult.didOverflowBounds() = didOverflowWidth || didOverflowHeight

    private fun TextLayoutResult.didOverflowByEllipsize(): Boolean =
        when (lineCount) {
            0 -> false
            1 -> isLineEllipsized(0)
            else ->
                when (layoutInput.overflow) {
                    TextOverflow.StartEllipsis,
                    TextOverflow.MiddleEllipsis -> didOverflowBounds()
                    TextOverflow.Ellipsis -> isLineEllipsized(lineCount - 1)
                    else -> false
                }
        }

    override fun equals(other: Any?): Boolean {
        if (other === this) return true
        if (other == null) return false
        if (other !is ActionTextAutoSize) return false

        if (other.minFontSize != minFontSize) return false
        if (other.maxFontSize != maxFontSize) return false
        if (other.stepSize != stepSize) return false
        if (other.layoutSize != layoutSize) return false

        return true
    }

    override fun hashCode(): Int {
        var result = minFontSize.hashCode()
        result = 31 * result + maxFontSize.hashCode()
        result = 31 * result + stepSize.hashCode()
        result = 31 * result + layoutSize.hashCode()
        return result
    }

}

@Preview(locale = "ru", device = Devices.PIXEL)
@Preview(locale = "ru", device = Devices.PIXEL_8)
@Preview(locale = "ru", device = Devices.PIXEL_9)
@Composable
fun PreviewAssetHeadActions() {
    WalletTheme {
        AssetHeadActions(
            walletType = WalletType.Multicoin,
            onTransfer = { },
            transferEnabled = true,
            operationsEnabled = true,
            onReceive = { },
            onBuy = {}
        ) {

        }
    }
}
