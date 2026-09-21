@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.features.settings.settings.viewmodels.models.settingsAction
import com.gemwallet.android.ui.BuildConfig
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import com.gemwallet.android.ui.theme.space0

@Composable
fun SettingsScene(onAction: (SettingsSceneAction) -> Unit, walletConnectEnabled: Boolean = true, scrollState: ScrollState = rememberScrollState()) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    val notificationsAvailable = viewModel.notificationsAvailable

    LaunchedEffect(walletConnectEnabled) { viewModel.setWalletConnectAvailable(walletConnectEnabled) }

    LifecycleResumeEffect(Unit) {
        viewModel.refreshDeveloperMode()
        onPauseOrDispose { }
    }

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
                    GemListRowView(
                        row = row,
                        listPosition = ListPosition.getPosition(index, section.rows.size),
                        modifier = Modifier.clickable { action?.let(onRowAction) },
                    )
                }
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }
}
