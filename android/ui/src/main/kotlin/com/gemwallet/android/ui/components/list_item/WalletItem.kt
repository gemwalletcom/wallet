package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.iconModel
import com.gemwallet.android.ui.style.supportIcon
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space0
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSubtitle

@Composable
fun WalletItem(row: GemWalletRow, isCurrent: Boolean, modifier: Modifier = Modifier, listPosition: ListPosition, onEdit: ((String) -> Unit)? = null) {
    val context = LocalContext.current
    ListItem(
        modifier = modifier,
        minHeight = ListItemDefaults.iconMinHeight,
        titleSubtitleSpacing = space0,
        trailingContentEndPadding = paddingSmall,
        leading = @Composable {
            IconWithBadge(
                icon = row.iconModel(context),
                supportIcon = row.supportIcon(),
            )
        },
        title = {
            ListItemTitleText(text = row.name)
        },
        subtitle = {
            ListItemSupportText(row.subtitle.string(context))
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
                    WalletEditButton(onClick = { onEdit(row.id) })
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
            row = GemWalletRow(
                id = "1",
                name = "Foo wallet name",
                subtitle = GemWalletSubtitle.Multicoin,
                placeholder = GemWalletPlaceholder.Multicoin,
                showsWatchBadge = false,
                isPinned = false,
                isCurrent = true,
                hasAvatar = false,
                imageUrl = null,
                deletePrompt = GemLocalizedText.DeleteConfirmation("Foo wallet name"),
            ),
            listPosition = ListPosition.Single,
            isCurrent = true,
            onEdit = {},
        )
    }
}
