package com.gemwallet.android.features.settings.presents.security

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.security.SecurityViewModel

@Composable
fun SecurityScreen(onCancel: () -> Unit, viewModel: SecurityViewModel = hiltViewModel()) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val lockInterval by viewModel.lockInterval.collectAsStateWithLifecycle(null)
    val error by viewModel.error.collectAsStateWithLifecycle()
    val isUpdatingAuthentication by viewModel.isUpdatingAuthentication.collectAsStateWithLifecycle()

    SecurityScene(
        sections = sections,
        lockInterval = lockInterval,
        lockPeriods = viewModel.lockPeriods,
        error = error,
        isUpdatingAuthentication = isUpdatingAuthentication,
        onErrorShown = viewModel::clearError,
        onAuthRequired = viewModel::setAuthRequired,
        onHideBalances = viewModel::setHideBalances,
        onLockInterval = { viewModel.setLockInterval(it) },
        onCancel = onCancel,
    )
}
