package com.gemwallet.android.features.nft.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.sp
import com.gemwallet.android.features.nft.viewmodels.models.NftActionUIModel
import com.gemwallet.android.ui.components.list_head.AmountHeadAction
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.paddingDefault
import uniffi.gemstone.GemCollectibleAction
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemHeaderButtonKind

@Composable
fun NftHeaderActions(header: GemHeaderActions, actions: List<NftActionUIModel>, onSend: () -> Unit, onAction: (GemCollectibleAction) -> Unit) {
    var actionFontSize by remember { mutableStateOf(16.sp) }
    var isMenuExpanded by remember { mutableStateOf(false) }
    val buttons = (header as? GemHeaderActions.Buttons)?.buttons.orEmpty()

    Row(
        modifier = Modifier.width(IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        buttons.forEach { button ->
            val title = stringResource(button.kind.stringRes())
            when (button.kind) {
                GemHeaderButtonKind.MORE -> Box(modifier = Modifier.weight(1f)) {
                    AmountHeadAction(
                        modifier = Modifier.fillMaxWidth(),
                        title = title,
                        imageVector = AppIcons.MoreVert,
                        contentDescription = title,
                        enabled = button.isEnabled,
                        fontSize = actionFontSize,
                        onNextFontSize = {
                            if (actionFontSize > it) actionFontSize = it
                        },
                        onClick = { isMenuExpanded = true },
                    )
                    DropdownMenu(
                        expanded = isMenuExpanded,
                        onDismissRequest = { isMenuExpanded = false },
                    ) {
                        actions.forEach { item ->
                            val color = if (item.isDestructive) MaterialTheme.colorScheme.error else Color.Unspecified
                            DropdownMenuItem(
                                text = { Text(item.title, color = color) },
                                leadingIcon = { Icon(item.action.icon(), contentDescription = null, tint = if (item.isDestructive) color else LocalContentColor.current) },
                                onClick = {
                                    isMenuExpanded = false
                                    onAction(item.action)
                                },
                            )
                        }
                    }
                }

                else -> AmountHeadAction(
                    modifier = Modifier.weight(1f),
                    title = title,
                    imageVector = AppIcons.Send,
                    contentDescription = title,
                    enabled = button.isEnabled,
                    fontSize = actionFontSize,
                    onNextFontSize = {
                        if (actionFontSize > it) actionFontSize = it
                    },
                    onClick = onSend,
                )
            }
        }
    }
}

@Composable
private fun GemCollectibleAction.icon(): ImageVector = when (this) {
    GemCollectibleAction.SAVE_IMAGE -> AppIcons.Image
    GemCollectibleAction.SET_AVATAR -> AppIcons.Wallet
    GemCollectibleAction.REFRESH -> AppIcons.Refresh
    GemCollectibleAction.REPORT -> AppIcons.Warning
}
