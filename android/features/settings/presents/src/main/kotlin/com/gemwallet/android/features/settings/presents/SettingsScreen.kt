package com.gemwallet.android.features.settings.presents

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.models.actions.SettingsAction

@Composable
fun SettingsScreen(onAction: (SettingsAction) -> Unit, scrollState: ScrollState = rememberScrollState()) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val sections by viewModel.sections.collectAsStateWithLifecycle()

    LifecycleResumeEffect(Unit) {
        viewModel.refreshDeveloperMode()
        onPauseOrDispose { }
    }

    SettingsScene(sections = sections, onAction = onAction, scrollState = scrollState)
}
