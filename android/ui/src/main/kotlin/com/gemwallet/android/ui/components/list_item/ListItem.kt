package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

object ListItemDefaults {
    val plainMinHeight: Dp = 56.dp
    val defaultMinHeight: Dp = 72.dp
    val supportingContentMinHeight: Dp = 88.dp
}

@Composable
fun ListItem(
    modifier: Modifier = Modifier,
    listPosition: ListPosition,
    minHeight: Dp = ListItemDefaults.defaultMinHeight,
    contentPadding: Dp = paddingMiddle,
    trailingContentEndPadding: Dp? = null,
    leading: (@Composable RowScope.() -> Unit)? = null,
    title: (@Composable () -> Unit)? = null,
    subtitle: (@Composable () -> Unit)? = null,
    trailing: (@Composable RowScope.() -> Unit)? = null,
) {
    val sidePadding = adaptivePadding(default = paddingDefault, compact = paddingSmall)
    val contentSpacing = paddingMiddle
    val trailingEndPadding = trailingContentEndPadding ?: defaultTrailingContentEndPadding(sidePadding)

    Row(
        modifier = Modifier
            .listItem(position = listPosition, paddingHorizontal = sidePadding)
            .then(
                modifier
                    .fillMaxWidth()
                    .padding(start = contentSpacing)
            ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(contentSpacing)
    ) {
        leading?.invoke(this)
        Row(
            modifier = Modifier
                .heightIn(min = minHeight)
                .padding(
                    top = contentPadding,
                    end = trailingEndPadding,
                    bottom = contentPadding,
                )
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(
                modifier = Modifier.weight(1f),
                horizontalAlignment = Alignment.Start,
                verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
            ) {
                title?.invoke()
                subtitle?.invoke()
            }
            trailing?.let {
                Row(
                    modifier = Modifier.wrapContentWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Spacer(modifier = Modifier.width(contentSpacing))
                    it.invoke(this)
                }
            }
        }
    }
}

internal fun defaultTrailingContentEndPadding(sidePadding: Dp): Dp {
    return sidePadding.coerceAtLeast(paddingDefault)
}

@Preview
@Composable
fun PreviewListItem() {
    MaterialTheme {
        ListItem(
            title = {
                Text(
                    "Some title",
                    overflow = TextOverflow.MiddleEllipsis,
                    maxLines = 1,
                )
            },
            listPosition = ListPosition.Single,
            trailing = {
                Text(
                    "Some_data_Some_data_Some_data_Some_data_Some_data_Some_data_Some_data_Some_data!",
                    overflow = TextOverflow.MiddleEllipsis,
                    maxLines = 1,
                )
            }
        )
    }
}
