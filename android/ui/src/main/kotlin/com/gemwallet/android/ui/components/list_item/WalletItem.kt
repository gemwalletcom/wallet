package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space0

@Composable
fun WalletItem(model: WalletRowUIModel, isCurrent: Boolean, modifier: Modifier = Modifier, listPosition: ListPosition, onEdit: ((String) -> Unit)? = null) {
    ListItem(
        modifier = modifier,
        minHeight = ListItemDefaults.iconMinHeight,
        titleSubtitleSpacing = space0,
        trailingContentEndPadding = paddingSmall,
        leading = @Composable {
            IconWithBadge(
                icon = model.icon,
                supportIcon = model.supportIcon,
            )
        },
        title = {
            ListItemTitleText(text = model.name)
        },
        subtitle = {
            ListItemSupportText(model.subtitle)
        },
        listPosition = listPosition,
        trailing = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Spacer16()
                if (isCurrent) {
                    SelectionCheckmark()
                }
                if (onEdit != null) {
                    Spacer8()
                    WalletEditButton(onClick = { onEdit(model.id) })
                }
            }
        },
    )
}

@Composable
private fun WalletEditButton(onClick: () -> Unit) {
    IconButton(onClick = onClick) {
        Icon(
            imageVector = AppIcons.SettingsOutlined,
            contentDescription = "edit",
            tint = MaterialTheme.colorScheme.secondary,
        )
    }
}

@Preview
@Composable
fun PreviewWalletItem() {
    MaterialTheme {
        WalletItem(
            model = WalletRowUIModel(
                id = "1",
                name = "Foo wallet name",
                subtitle = "Multicoin",
                icon = R.drawable.multicoin_wallet,
                supportIcon = null,
            ),
            listPosition = ListPosition.Single,
            isCurrent = true,
            onEdit = {},
        )
    }
}
