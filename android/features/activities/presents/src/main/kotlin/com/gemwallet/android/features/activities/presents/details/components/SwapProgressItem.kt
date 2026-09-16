package com.gemwallet.android.features.activities.presents.details.components

import androidx.annotation.StringRes
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DividerDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.features.activities.presents.localization.stringRes
import com.gemwallet.android.features.activities.presents.style.icon
import uniffi.gemstone.GemSwapProgressMarker
import uniffi.gemstone.GemSwapProgressState
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemValueStyle

private val connectorWidth = 1.5.dp

@Composable
internal fun SwapProgressItem(progress: TransactionDetailsValue.SwapProgress) {
    val chainName = progress.fromAsset.chain.networkName()
    val transferValue = ValueFormatter(style = GemValueStyle.AUTO)
        .string(progress.fromValue, progress.fromAsset)

    val transferState = progress.transfer
    val swapState = progress.swap
    val estimatedTime = progress.etaInSeconds?.let(::formatEstimatedConfirmation)

    Row(
        modifier = Modifier
            .listItem(ListPosition.Single)
            .fillMaxWidth()
            .padding(horizontal = ListItemDefaults.contentSpacing, vertical = paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(ListItemDefaults.contentSpacing),
        verticalAlignment = Alignment.Top,
    ) {
        Timeline(
            transferState = transferState,
            swapState = swapState,
        )
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(space8),
        ) {
            ProgressStep(
                title = stringResource(R.string.transfer_title),
                subtitle = "$transferValue ($chainName)",
                state = transferState,
                estimatedTime = estimatedTime,
            )
            ProgressStep(
                title = stringResource(R.string.wallet_swap),
                subtitle = progress.providerName,
                state = swapState,
                estimatedTime = estimatedTime,
            )
        }
    }
}

@Composable
private fun ProgressStep(
    title: String,
    subtitle: String,
    state: GemSwapProgressState,
    estimatedTime: String?,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(space6),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(space8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                modifier = Modifier.weight(1f),
                text = title,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
            )
            StatusTag(state = state)
        }
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(space8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                modifier = Modifier.weight(1f),
                text = subtitle,
                color = MaterialTheme.colorScheme.secondary,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium,
            )
            estimatedTime?.takeIf { state.marker == GemSwapProgressMarker.SPINNER }?.let {
                Text(
                    text = it,
                    color = MaterialTheme.colorScheme.secondary,
                    maxLines = 1,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}

@Composable
private fun Timeline(
    transferState: GemSwapProgressState,
    swapState: GemSwapProgressState,
) {
    Column(
        modifier = Modifier.width(iconSize),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val connectorColor = when (transferState.step) {
            GemSwapProgressStep.COMPLETED -> MaterialTheme.colorScheme.tertiary
            GemSwapProgressStep.PENDING,
            GemSwapProgressStep.WAITING,
            GemSwapProgressStep.FAILED,
            GemSwapProgressStep.REVERTED,
            GemSwapProgressStep.REFUNDED -> MaterialTheme.colorScheme.outlineVariant
        }

        ProgressMarker(transferState)
        Connector(color = connectorColor)
        ProgressMarker(swapState)
    }
}

@Composable
private fun Connector(color: Color) {
    Box(
        modifier = Modifier
            .width(connectorWidth)
            .height(space24)
            .background(color),
    )
}

@Composable
private fun ProgressMarker(state: GemSwapProgressState) {
    val color = state.step.color()
    val markerModifier = Modifier
        .size(iconSize)
        .then(
            when (state.marker) {
                GemSwapProgressMarker.CHECK,
                GemSwapProgressMarker.CROSS,
                GemSwapProgressMarker.SWAP -> Modifier.background(color.copy(alpha = alpha10), CircleShape)
                GemSwapProgressMarker.SPINNER,
                GemSwapProgressMarker.DOTS -> Modifier
            }
        )
        .border(DividerDefaults.Thickness, color, CircleShape)

    Box(
        modifier = markerModifier,
        contentAlignment = Alignment.Center,
    ) {
        when (state.marker) {
            GemSwapProgressMarker.SPINNER -> CircularProgressIndicator16(color = color)
            GemSwapProgressMarker.DOTS -> Row(
                horizontalArrangement = Arrangement.spacedBy(space2),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                repeat(3) {
                    Box(
                        modifier = Modifier
                            .size(space4)
                            .background(color, CircleShape),
                    )
                }
            }
            GemSwapProgressMarker.CHECK,
            GemSwapProgressMarker.CROSS,
            GemSwapProgressMarker.SWAP -> state.marker.icon()?.let { icon ->
                Icon(
                    modifier = Modifier.size(compactIconSize),
                    imageVector = icon,
                    contentDescription = null,
                    tint = color,
                )
            }
        }
    }
}

@Composable
private fun StatusTag(state: GemSwapProgressState) {
    val labelRes = state.step.stringRes() ?: return
    val color = state.step.color()
    Text(
        modifier = Modifier
            .background(color = color.copy(alpha = alpha10), shape = RoundedCornerShape(space6))
            .padding(horizontal = space8, vertical = space2),
        text = stringResource(labelRes),
        color = color,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        style = MaterialTheme.typography.labelMedium,
    )
}

@Composable
private fun GemSwapProgressStep.color(): Color {
    return when (this) {
        GemSwapProgressStep.COMPLETED -> MaterialTheme.colorScheme.tertiary
        GemSwapProgressStep.PENDING -> MaterialTheme.colorScheme.primary
        GemSwapProgressStep.WAITING -> MaterialTheme.colorScheme.outlineVariant
        GemSwapProgressStep.FAILED -> MaterialTheme.colorScheme.error
        GemSwapProgressStep.REVERTED -> MaterialTheme.colorScheme.error
        GemSwapProgressStep.REFUNDED -> pendingColor
    }
}
