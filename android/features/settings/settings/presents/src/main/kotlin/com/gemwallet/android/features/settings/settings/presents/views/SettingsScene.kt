@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.ScrollState
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
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.features.settings.settings.presents.localization.stringRes
import com.gemwallet.android.features.settings.settings.presents.style.action
import com.gemwallet.android.features.settings.settings.presents.style.icon
import uniffi.gemstone.GemSettingsRow

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SettingsScene(
    onAction: (SettingsSceneAction) -> Unit,
    walletConnectEnabled: Boolean = true,
    scrollState: ScrollState = rememberScrollState()
) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val walletsCount by viewModel.walletsCount.collectAsStateWithLifecycle()
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    var isShowDevelopEnable by remember { mutableStateOf(false) }
    var requestPushGrant by remember { mutableStateOf<(() -> Unit)?>(null) }
    val notificationsAvailable = viewModel.notificationsAvailable

    LaunchedEffect(walletConnectEnabled) { viewModel.setWalletConnectAvailable(walletConnectEnabled) }

    val onSupport = {
        if (notificationsAvailable && !pushEnabled) {
            requestPushGrant = {
                viewModel.enableNotifications()
                onAction(SettingsSceneAction.Support)
            }
        } else {
            onAction(SettingsSceneAction.Support)
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
            sections.forEach { section ->
                section.rows.forEachIndexed { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.rows.size)
                    when (row) {
                        GemSettingsRow.WALLETS -> LinkItem(
                            title = stringResource(row.stringRes()),
                            icon = row.icon(),
                            listPosition = listPosition,
                            trailingContent = {
                                PropertyDataText(
                                    text = walletsCount.toString(),
                                    badge = { DataBadgeChevron() },
                                )
                            },
                            onClick = { onAction(SettingsSceneAction.Wallets) },
                        )
                        GemSettingsRow.SUPPORT -> LinkItem(
                            title = stringResource(row.stringRes()),
                            icon = row.icon(),
                            listPosition = listPosition,
                            onClick = { onSupport() },
                        )
                        GemSettingsRow.ABOUT_US -> Box(modifier = Modifier.fillMaxWidth()) {
                            LinkItem(
                                title = stringResource(row.stringRes()),
                                icon = row.icon(),
                                listPosition = listPosition,
                                onClick = { onAction(SettingsSceneAction.AboutUs) },
                                onLongClick = { isShowDevelopEnable = true },
                            )
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
                        GemSettingsRow.SECURITY,
                        GemSettingsRow.NOTIFICATIONS,
                        GemSettingsRow.PREFERENCES,
                        GemSettingsRow.WALLET_CONNECT,
                        GemSettingsRow.REWARDS,
                        GemSettingsRow.DEVELOPER -> LinkItem(
                            title = stringResource(row.stringRes()),
                            icon = row.icon(),
                            listPosition = listPosition,
                            onClick = { onAction(row.action()) },
                        )
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
