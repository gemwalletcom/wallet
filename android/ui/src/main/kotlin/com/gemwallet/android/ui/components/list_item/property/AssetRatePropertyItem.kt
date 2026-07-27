package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.foundation.clickable
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4

@Composable
fun AssetRatePropertyItem(
    rate: AssetRatePair,
    listPosition: ListPosition,
) {
    var showReverse by remember { mutableStateOf(false) }
    val displayedRate = if (showReverse) rate.reverse else rate.forward

    PropertyItem(
        modifier = Modifier.clickable { showReverse = !showReverse },
        title = { PropertyTitleText(R.string.buy_rate) },
        data = {
            PropertyDataText(
                text = displayedRate,
                badge = {
                    Spacer4()
                    Icon(
                        modifier = Modifier.clip(MaterialTheme.shapes.small),
                        imageVector = AppIcons.SwapVert,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary,
                    )
                },
            )
        },
        listPosition = listPosition,
    )
}
