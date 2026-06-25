package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.features.confirm.viewmodels.SimulationAssetChange
import com.gemwallet.android.features.confirm.viewmodels.formattedValue
import com.gemwallet.android.features.confirm.viewmodels.valueDirection
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun ConfirmBalanceChanges(changes: List<SimulationAssetChange>) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(paddingDefault))
            .background(MaterialTheme.colorScheme.background),
    ) {
        changes.forEachIndexed { index, change ->
            if (index > 0) {
                HorizontalDivider(modifier = Modifier.padding(start = paddingDefault))
            }
            ConfirmBalanceChangeRow(change)
        }
    }
}

@Composable
private fun ConfirmBalanceChangeRow(change: SimulationAssetChange) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = paddingDefault, vertical = paddingSmall),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AssetIcon(asset = change.asset, size = listItemIconSize)
        Spacer(modifier = Modifier.width(paddingMiddle))
        Text(
            modifier = Modifier.weight(1f),
            text = change.asset.name,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
        )
        Spacer(modifier = Modifier.width(paddingMiddle))
        Text(
            text = change.formattedValue(),
            style = MaterialTheme.typography.bodyLarge,
            color = change.valueDirection().color(),
            textAlign = TextAlign.End,
            maxLines = 1,
        )
    }
}
