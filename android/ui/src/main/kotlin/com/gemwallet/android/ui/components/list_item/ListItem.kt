package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
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
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle

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
    leading: (@Composable RowScope.() -> Unit)? = null,
    title: (@Composable () -> Unit)? = null,
    subtitle: (@Composable () -> Unit)? = null,
    trailing: (@Composable RowScope.() -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .listItem(position = listPosition)
            .then(
                modifier
                    .fillMaxWidth()
                    .padding(start = paddingDefault)
            ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingDefault)
    ) {
        leading?.invoke(this)
        Row(
            modifier = Modifier
                .heightIn(min = minHeight)
                .padding(top = paddingMiddle, end = paddingDefault, bottom = paddingMiddle)
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
                    Spacer16()
                    it.invoke(this)
                }
            }
        }
    }
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
