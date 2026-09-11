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
import com.gemwallet.android.ui.icons.AppIcons
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
import uniffi.gemstone.GemSwapProgressStep

private val connectorWidth = 1.5.dp

@Composable
internal fun SwapProgressItem(progress: TransactionDetailsValue.SwapProgress) {
    val chainName = progress.fromAsset.chain.networkName()
    val transferValue = ValueFormatter(style = ValueFormatter.Style.Auto)
        .string(progress.fromValue, progress.fromAsset)

    val transferStatus = progress.transfer
    val swapStatus = progress.swap
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
            transferStatus = transferStatus,
            swapStatus = swapStatus,
        )
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(space8),
        ) {
            ProgressStep(
                title = stringResource(R.string.transfer_title),
                subtitle = "$transferValue ($chainName)",
                status = transferStatus,
                estimatedTime = estimatedTime,
            )
            ProgressStep(
                title = stringResource(R.string.wallet_swap),
                subtitle = progress.providerName,
                status = swapStatus,
                estimatedTime = estimatedTime,
            )
        }
    }
}

@Composable
private fun ProgressStep(
    title: String,
    subtitle: String,
    status: GemSwapProgressStep,
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
            StatusTag(status = status)
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
            estimatedTime?.takeIf { status == GemSwapProgressStep.PENDING }?.let {
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
    transferStatus: GemSwapProgressStep,
    swapStatus: GemSwapProgressStep,
) {
    Column(
        modifier = Modifier.width(iconSize),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val connectorColor = when (transferStatus) {
            GemSwapProgressStep.COMPLETED -> MaterialTheme.colorScheme.tertiary
            GemSwapProgressStep.PENDING,
            GemSwapProgressStep.WAITING,
            GemSwapProgressStep.FAILED,
            GemSwapProgressStep.REVERTED,
            GemSwapProgressStep.REFUNDED -> MaterialTheme.colorScheme.outlineVariant
        }

        ProgressMarker(transferStatus)
        Connector(color = connectorColor)
        ProgressMarker(swapStatus)
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
private fun ProgressMarker(status: GemSwapProgressStep) {
    val color = status.color()
    val markerModifier = Modifier
        .size(iconSize)
        .then(
            when (status) {
                GemSwapProgressStep.COMPLETED,
                GemSwapProgressStep.FAILED,
                GemSwapProgressStep.REVERTED,
                GemSwapProgressStep.REFUNDED -> Modifier.background(color.copy(alpha = alpha10), CircleShape)
                GemSwapProgressStep.PENDING,
                GemSwapProgressStep.WAITING -> Modifier
            }
        )
        .border(DividerDefaults.Thickness, color, CircleShape)

    Box(
        modifier = markerModifier,
        contentAlignment = Alignment.Center,
    ) {
        when (status) {
            GemSwapProgressStep.COMPLETED -> Icon(
                modifier = Modifier.size(compactIconSize),
                imageVector = AppIcons.Check,
                contentDescription = null,
                tint = color,
            )
            GemSwapProgressStep.PENDING -> CircularProgressIndicator16(color = color)
            GemSwapProgressStep.WAITING -> Row(
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
            GemSwapProgressStep.FAILED,
            GemSwapProgressStep.REVERTED,
            GemSwapProgressStep.REFUNDED -> Icon(
                modifier = Modifier.size(compactIconSize),
                imageVector = AppIcons.Close,
                contentDescription = null,
                tint = color,
            )
        }
    }
}

@Composable
private fun StatusTag(status: GemSwapProgressStep) {
    val labelRes = status.labelRes() ?: return
    val color = status.color()
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

@StringRes
internal fun GemSwapProgressStep.labelRes(): Int? {
    return when (this) {
        GemSwapProgressStep.COMPLETED -> R.string.transaction_status_completed
        GemSwapProgressStep.PENDING -> R.string.transaction_status_inprogress
        GemSwapProgressStep.WAITING -> null
        GemSwapProgressStep.FAILED -> R.string.transaction_status_failed
        GemSwapProgressStep.REVERTED -> R.string.transaction_status_reverted
        GemSwapProgressStep.REFUNDED -> R.string.transaction_status_refunded
    }
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
