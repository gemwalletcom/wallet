package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.contacts.viewmodels.ContactsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.secondaryFaded
import com.wallet.core.primitives.Contact

@Composable
fun ContactsNavScreen(
    onAction: (ContactsAction) -> Unit,
    viewModel: ContactsViewModel = hiltViewModel(),
) {
    val contacts by viewModel.contacts.collectAsStateWithLifecycle()
    val revealed = remember { mutableStateOf<String?>(null) }

    Scene(
        title = stringResource(R.string.contacts_title),
        onClose = { onAction(ContactsAction.Cancel) },
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
                                    viewModel.deleteContact(item.contact.id)
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
                        ContactListItem(
                            contact = item.contact,
                            listPosition = itemPosition,
                            onClick = { onAction(ContactsAction.OpenContact(item.contact.id)) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ContactListItem(
    contact: Contact,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        minHeight = ListItemDefaults.defaultMinHeight,
        leading = { ContactAvatar(name = contact.name) },
        title = { ListItemTitleText(text = contact.name) },
        subtitle = contact.description?.takeIf { it.isNotBlank() }?.let { description ->
            {
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.secondary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        },
        trailing = { DataBadgeChevron() },
    )
}

@Composable
private fun ContactAvatar(
    name: String,
    size: Dp = listItemIconSize,
) {
    Box(
        modifier = Modifier
            .size(size)
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.secondaryFaded),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = name.take(2).uppercase(),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}
