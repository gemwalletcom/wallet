package com.gemwallet.android.features.transactions.presents.transaction.components

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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionSwapProgressStepUIModel
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionSwapProgressUIModel
import com.gemwallet.android.ui.components.image.vector
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SwapProgressMarkerUIModel
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8

private val connectorWidth = 1.5.dp

@Composable
internal fun TransactionSwapProgressItem(progress: TransactionSwapProgressUIModel) {
    Row(
        modifier = Modifier
            .listItem(ListPosition.Single)
            .fillMaxWidth()
            .padding(horizontal = ListItemDefaults.contentSpacing, vertical = paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(ListItemDefaults.contentSpacing),
        verticalAlignment = Alignment.Top,
    ) {
        Timeline(progress)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(space8),
        ) {
            ProgressStep(progress.transfer, progress.estimatedTime)
            ProgressStep(progress.swap, progress.estimatedTime)
        }
    }
}

@Composable
private fun ProgressStep(step: TransactionSwapProgressStepUIModel, estimatedTime: String?) {
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
                text = step.title,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
            )
            StatusTag(step)
        }
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(space8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                modifier = Modifier.weight(1f),
                text = step.subtitle,
                color = MaterialTheme.colorScheme.secondary,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium,
            )
            estimatedTime?.takeIf { step.showsEstimatedTime }?.let {
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
private fun Timeline(progress: TransactionSwapProgressUIModel) {
    Column(
        modifier = Modifier.width(iconSize),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val connectorColor = if (progress.isConnectorActive) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.outlineVariant

        ProgressMarker(progress.transfer)
        Connector(color = connectorColor)
        ProgressMarker(progress.swap)
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
private fun ProgressMarker(step: TransactionSwapProgressStepUIModel) {
    val color = step.style.color()
    val markerModifier = Modifier
        .size(iconSize)
        .then(
            when (step.marker) {
                is SwapProgressMarkerUIModel.Icon -> Modifier.background(color.copy(alpha = alpha10), CircleShape)

                SwapProgressMarkerUIModel.Spinner,
                SwapProgressMarkerUIModel.Dots,
                -> Modifier
            },
        )
        .border(DividerDefaults.Thickness, color, CircleShape)

    Box(
        modifier = markerModifier,
        contentAlignment = Alignment.Center,
    ) {
        when (val marker = step.marker) {
            SwapProgressMarkerUIModel.Spinner -> CircularProgressIndicator16(color = color)

            SwapProgressMarkerUIModel.Dots -> Row(
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

            is SwapProgressMarkerUIModel.Icon -> Icon(
                modifier = Modifier.size(compactIconSize),
                imageVector = marker.symbol.vector(),
                contentDescription = null,
                tint = color,
            )
        }
    }
}

@Composable
private fun StatusTag(step: TransactionSwapProgressStepUIModel) {
    val label = step.statusLabel ?: return
    val color = step.style.color()
    Text(
        modifier = Modifier
            .background(color = color.copy(alpha = alpha10), shape = RoundedCornerShape(space6))
            .padding(horizontal = space8, vertical = space2),
        text = label,
        color = color,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        style = MaterialTheme.typography.labelMedium,
    )
}
