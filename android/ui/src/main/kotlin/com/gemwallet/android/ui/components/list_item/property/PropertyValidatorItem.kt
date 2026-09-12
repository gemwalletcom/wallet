package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.formatApr
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingMiddle
import uniffi.gemstone.GemValidatorRow

@Composable
fun PropertyValidatorItem(
    validator: GemValidatorRow,
    listPosition: ListPosition = ListPosition.Single,
    onClick: (() -> Unit)? = null,
) {
    PropertyItem(
        modifier = Modifier.clickable(enabled = onClick != null) { onClick?.invoke() },
        title = {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(paddingMiddle),
            ) {
                IconWithBadge(
                    icon = validator.imageUrl,
                    placeholder = validator.placeholder,
                )
                Text(
                    modifier = Modifier.weight(1f),
                    text = validator.name,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
        },
        data = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                ListItemSupportText(R.string.stake_apr, " ${validator.validator.apr.formatApr()}")
                if (onClick != null) {
                    DataBadgeChevron()
                }
            }
        },
        listPosition = listPosition,
    )
}
