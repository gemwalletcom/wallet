package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
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
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.contacts.viewmodels.ContactsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.icons.AppIcons

@Composable
fun ContactsNavScreen(
    onAction: (ContactsAction) -> Unit,
    viewModel: ContactsViewModel = hiltViewModel(),
) {
    val contacts by viewModel.contacts.collectAsStateWithLifecycle()
    val errorText by viewModel.errorText.collectAsStateWithLifecycle()
    val revealed = remember { mutableStateOf<String?>(null) }
    val snackbar = rememberSnackbarState(message = errorText, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    Scene(
        title = stringResource(R.string.contacts_title),
        onClose = { onAction(ContactsAction.Cancel) },
        snackbar = snackbar,
        actions = {
            IconButton(onClick = { onAction(ContactsAction.AddContact) }) {
                Icon(imageVector = AppIcons.Add, contentDescription = "")
            }
        },
    ) {
        if (contacts.isEmpty()) {
            EmptyContentView(
                type = EmptyContentType.Contacts,
                modifier = Modifier.fillMaxSize(),
            )
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                itemsPositioned(contacts, key = { _, item -> item.contact.id }) { position, item ->
                    SwipeableItemWithActions(
                        isRevealed = revealed.value == item.contact.id,
                        actions = {
                            ActionIcon(
                                onClick = {
                                    viewModel.deleteContact(item.contact)
                                    revealed.value = null
                                },
                                backgroundColor = MaterialTheme.colorScheme.error,
                                icon = AppIcons.Delete,
                            )
                        },
                        onExpanded = { revealed.value = item.contact.id },
                        onCollapsed = { revealed.value = null },
                        listPosition = position,
                    ) { itemPosition ->
                        ListItem(
                            model = remember(item) { viewModel.listItem(item) },
                            listPosition = itemPosition,
                            modifier = Modifier.clickable { onAction(ContactsAction.OpenContact(item.contact.id)) },
                            minHeight = ListItemDefaults.defaultMinHeight,
                            accessory = { DataBadgeChevron() },
                        )
                    }
                }
            }
        }
    }
}
