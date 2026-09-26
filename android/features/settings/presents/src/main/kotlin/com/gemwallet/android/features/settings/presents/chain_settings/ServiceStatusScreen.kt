package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.chain_settings.ServiceStatusViewModel
import com.gemwallet.android.ui.LocalStreamConnected

@Composable
fun ServiceStatusScreen(onCancel: () -> Unit, viewModel: ServiceStatusViewModel = hiltViewModel()) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val isStreamConnected by LocalStreamConnected.current.collectAsStateWithLifecycle()

    LaunchedEffect(isStreamConnected) { viewModel.fetch() }

    ServiceStatusScene(sections = sections, onRefresh = viewModel::fetch, onCancel = onCancel)
}
