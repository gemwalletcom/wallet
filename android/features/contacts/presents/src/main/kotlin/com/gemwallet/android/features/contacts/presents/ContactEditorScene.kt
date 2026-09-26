package com.gemwallet.android.features.contacts.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.contacts.viewmodels.models.ContactEditorUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
fun ContactEditorScene(state: ContactEditorUIState, snackbar: SnackbarHostState? = null, onNameChange: (String) -> Unit, onDescriptionChange: (String) -> Unit, onAction: (ContactEditorAction) -> Unit) {
    val revealed = remember { mutableStateOf<String?>(null) }

    Scene(
        title = stringResource(R.string.contacts_contact),
        onClose = { onAction(ContactEditorAction.Cancel) },
        snackbar = snackbar,
        actions = {
            IconButton(onClick = { onAction(ContactEditorAction.Save) }, enabled = state.isSaveEnabled) {
                Icon(imageVector = AppIcons.Check, contentDescription = "")
            }
        },
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = paddingDefault),
                    contentAlignment = Alignment.Center,
                ) {
                    ContactAvatar(
                        image = state.avatar,
                        size = extraLargeIconSize,
                        modifier = Modifier.clickable { onAction(ContactEditorAction.SelectAvatar) },
                        onRemove = if (state.hasAvatar) {
                            { onAction(ContactEditorAction.RemoveAvatar) }
                        } else {
                            null
                        },
                    )
                }
            }
            item {
                GemTextField(
                    value = state.name,
                    onValueChange = onNameChange,
                    label = stringResource(R.string.wallet_name),
                    listPosition = ListPosition.First,
                )
            }
            item {
                GemTextField(
                    value = state.description,
                    onValueChange = onDescriptionChange,
                    label = stringResource(R.string.common_description),
                    listPosition = ListPosition.Last,
                )
            }

            item { SubheaderItem(title = stringResource(R.string.contacts_addresses)) }

            itemsIndexed(state.addressRows, key = { _, item -> item.address.id }) { index, row ->
                val address = row.address.toPrimitives()
                SwipeableItemWithActions(
                    isRevealed = revealed.value == row.address.id,
                    actions = {
                        ActionIcon(
                            onClick = {
                                onAction(ContactEditorAction.DeleteAddress(address))
                                revealed.value = null
                            },
                            backgroundColor = MaterialTheme.colorScheme.error,
                            icon = AppIcons.Delete,
                        )
                    },
                    onExpanded = { revealed.value = row.address.id },
                    onCollapsed = { revealed.value = null },
                    listPosition = if (index == 0) ListPosition.First else ListPosition.Middle,
                ) { position ->
                    ListItem(
                        model = ListItemModel(title = row.chain.title, titleExtra = row.shortAddress, image = ListItemImage.Asset(row.chain.icon)),
                        listPosition = position,
                        modifier = Modifier.clickable { onAction(ContactEditorAction.EditAddress(address)) },
                        accessory = { DataBadgeChevron() },
                    )
                }
            }

            item {
                ListItem(
                    model = ListItemModel(
                        title = stringResource(R.string.common_address),
                        titleStyle = ListItemTextStyle.Primary,
                        image = ListItemImage.Symbol(ListItemSymbol.Add, tint = ListItemTextStyle.Primary),
                    ),
                    listPosition = if (state.addressRows.isEmpty()) ListPosition.Single else ListPosition.Last,
                    modifier = Modifier.clickable { onAction(ContactEditorAction.AddAddress) },
                )
            }
        }
    }
}
