package com.gemwallet.android.features.activities.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.SwapVert
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4

@Composable
internal fun TransactionRateProperty(
    property: TransactionDetailsValue.Rate,
    position: ListPosition,
) {
    var showReverse by remember { mutableStateOf(false) }
    val displayedRate = if (showReverse) property.reverse else property.forward

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
                        imageVector = Icons.Default.SwapVert,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary,
                    )
                },
            )
        },
        listPosition = position,
    )
}
