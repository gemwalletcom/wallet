package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun NftListItem(
    model: NftItemUIModel,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        minHeight = ListItemDefaults.iconMinHeight,
        leading = {
            NftImage(
                source = model.toImageSource(),
                modifier = Modifier
                    .size(listItemIconSize)
                    .clip(RoundedCornerShape(paddingSmall)),
            )
        },
        title = {
            Text(
                text = model.name,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        trailing = {
            model.collectionSize?.let { count ->
                Text(
                    text = count.toString(),
                    color = MaterialTheme.colorScheme.secondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            ChevronIcon()
        },
    )
}
