package com.gemwallet.android.features.settings.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalConfiguration
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.PreferencesViewModel
import com.gemwallet.android.ui.models.actions.PreferencesAction

@Composable
fun PreferencesScreen(onAction: (PreferencesAction) -> Unit, viewModel: PreferencesViewModel = hiltViewModel()) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val appearance by viewModel.appearance.collectAsStateWithLifecycle()
    val perpetualDefaults by viewModel.perpetualDefaults.collectAsStateWithLifecycle()
    val configuration = LocalConfiguration.current

    LaunchedEffect(configuration) { viewModel.setLanguage(configuration) }

    PreferencesScene(
        sections = sections,
        appearance = appearance,
        perpetualDefaults = perpetualDefaults,
        perpetualOptions = viewModel.perpetualOptions,
        onAction = onAction,
        onAppearance = { viewModel.setAppearance(it) },
        onPerpetualEnabled = { viewModel.setPerpetualEnabled(it) },
        onPerpetualOption = { setting, value -> viewModel.setPerpetualOption(setting, value) },
    )
}
