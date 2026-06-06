package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.ContactAddress

@Composable
fun ManageContactScene(
    state: ManageContactUIState,
    onNameChange: (String) -> Unit,
    onDescriptionChange: (String) -> Unit,
    onAction: (ManageContactAction) -> Unit,
) {
    val revealed = remember { mutableStateOf<String?>(null) }

    Scene(
        title = stringResource(R.string.contacts_contact),
        onClose = { onAction(ManageContactAction.Cancel) },
        actions = {
            IconButton(onClick = { onAction(ManageContactAction.Save) }, enabled = state.isSaveEnabled) {
                Icon(imageVector = AppIcons.Check, contentDescription = "")
            }
        },
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
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

            itemsIndexed(state.addresses, key = { _, item -> item.id }) { index, address ->
                SwipeableItemWithActions(
                    isRevealed = revealed.value == address.id,
                    actions = {
                        ActionIcon(
                            onClick = {
                                onAction(ManageContactAction.DeleteAddress(address))
                                revealed.value = null
                            },
                            backgroundColor = MaterialTheme.colorScheme.error,
                            icon = AppIcons.Delete,
                        )
                    },
                    onExpanded = { revealed.value = address.id },
                    onCollapsed = { revealed.value = null },
                    listPosition = if (index == 0) ListPosition.First else ListPosition.Middle,
                ) { position ->
                    ContactAddressItem(
                        address = address,
                        listPosition = position,
                        onClick = { onAction(ManageContactAction.EditAddress(address)) },
                    )
                }
            }

            item {
                AddAddressItem(
                    listPosition = if (state.addresses.isEmpty()) ListPosition.Single else ListPosition.Last,
                    onClick = { onAction(ManageContactAction.AddAddress) },
                )
            }
        }
    }
}

@Composable
private fun ContactAddressItem(
    address: ContactAddress,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = { IconWithBadge(icon = address.chain.getIconUrl()) },
        title = { ListItemTitleText(text = address.chain.asset().name) },
        subtitle = {
            Text(
                text = AddressFormatter(address = address.address, chain = address.chain).value(),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.secondary,
            )
        },
        trailing = { DataBadgeChevron() },
    )
}

@Composable
private fun AddAddressItem(
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            Icon(
                imageVector = AppIcons.Add,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
            )
        },
        title = {
            Text(
                text = stringResource(R.string.common_address),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.primary,
            )
        },
    )
}
