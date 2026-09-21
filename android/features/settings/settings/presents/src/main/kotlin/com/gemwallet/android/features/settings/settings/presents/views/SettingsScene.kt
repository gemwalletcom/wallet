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
import com.gemwallet.android.features.settings.settings.viewmodels.models.opensDeveloperMenu
import com.gemwallet.android.features.settings.settings.viewmodels.models.settingsAction
import com.gemwallet.android.ui.BuildConfig
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import com.gemwallet.android.ui.theme.space0

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SettingsScene(onAction: (SettingsSceneAction) -> Unit, walletConnectEnabled: Boolean = true, scrollState: ScrollState = rememberScrollState()) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    val isDeveloperEnabled by viewModel.isDeveloperEnabled.collectAsStateWithLifecycle()
    var isShowDevelopEnable by remember { mutableStateOf(false) }
    val notificationsAvailable = viewModel.notificationsAvailable

    LaunchedEffect(walletConnectEnabled) { viewModel.setWalletConnectAvailable(walletConnectEnabled) }

    val onRowAction: (SettingsSceneAction) -> Unit = { action ->
        if (action == SettingsSceneAction.Support && notificationsAvailable && !pushEnabled) {
            viewModel.enableNotifications()
        }
        onAction(action)
    }

    Scene(
        title = stringResource(id = R.string.settings_title),
        mainActionPadding = PaddingValues(space0),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(scrollState),
        ) {
            sections.forEach { section ->
                section.rows.forEachIndexed { index, row ->
                    val action = row.settingsAction()
                    val opensDeveloperMenu = row.opensDeveloperMenu()
                    Box(modifier = Modifier.fillMaxWidth()) {
                        GemListRowView(
                            row = row,
                            listPosition = ListPosition.getPosition(index, section.rows.size),
                            modifier = Modifier.combinedClickable(
                                onClick = { action?.let(onRowAction) },
                                onLongClick = { isShowDevelopEnable = true }.takeIf { opensDeveloperMenu },
                            ),
                        )
                        if (opensDeveloperMenu) {
                            DropdownMenu(
                                isShowDevelopEnable,
                                { isShowDevelopEnable = false },
                                containerColor = MaterialTheme.colorScheme.background,
                            ) {
                                DropdownMenuItem(
                                    text = {
                                        Text(
                                            stringResource(
                                                if (isDeveloperEnabled) R.string.settings_disable_value else R.string.settings_enable_value,
                                                stringResource(R.string.settings_developer),
                                            ),
                                        )
                                    },
                                    onClick = {
                                        isShowDevelopEnable = false
                                        viewModel.toggleDeveloperMode()
                                    },
                                )
                            }
                        }
                    }
                }
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }
}
