@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.BuildConfig
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PushRequest
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import com.gemwallet.android.ui.theme.space0

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SettingsScene(
    onAction: (SettingsSceneAction) -> Unit,
    walletConnectEnabled: Boolean = true,
    scrollState: ScrollState = rememberScrollState()
) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val rows by viewModel.rows.collectAsStateWithLifecycle()
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    var isShowDevelopEnable by remember { mutableStateOf(false) }
    var requestPushGrant by remember { mutableStateOf<(() -> Unit)?>(null) }
    val notificationsAvailable = viewModel.notificationsAvailable

    LaunchedEffect(walletConnectEnabled) { viewModel.setWalletConnectAvailable(walletConnectEnabled) }

    val onRowAction: (SettingsSceneAction) -> Unit = { action ->
        if (action == SettingsSceneAction.Support && notificationsAvailable && !pushEnabled) {
            requestPushGrant = {
                viewModel.enableNotifications()
                onAction(action)
            }
        } else {
            onAction(action)
        }
    }

    Scene(
        title = stringResource(id = R.string.settings_title),
        mainActionPadding = PaddingValues(space0),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(scrollState)
        ) {
            rows.forEach { section ->
                section.items.forEachIndexed { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.items.size)
                    Box(modifier = Modifier.fillMaxWidth()) {
                        ListItem(
                            model = row.model,
                            listPosition = listPosition,
                            modifier = Modifier.combinedClickable(
                                onClick = { onRowAction(row.action) },
                                onLongClick = { isShowDevelopEnable = true }.takeIf { row.opensDeveloperMenu },
                            ),
                            minHeight = ListItemDefaults.plainMinHeight,
                            accessory = { DataBadgeChevron() },
                        )
                        if (row.opensDeveloperMenu) {
                            DropdownMenu(
                                isShowDevelopEnable, { isShowDevelopEnable = false },
                                containerColor = MaterialTheme.colorScheme.background,
                            ) {
                                DropdownMenuItem(
                                    text = { Text("Enable develop") },
                                    onClick = {
                                        isShowDevelopEnable = false
                                        viewModel.developEnable()
                                    }
                                )
                            }
                        }
                    }
                }
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }

    requestPushGrant?.let {
        PushRequest(
            onNotificationEnable = {
                it()
                requestPushGrant = null
            }
        ) { requestPushGrant = null }
    }
}
