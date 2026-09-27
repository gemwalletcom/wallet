package com.gemwallet.android.features.settings.presents.lock

import androidx.annotation.DrawableRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.lock.LockViewModel

@Composable
fun LockScreen(isContentReady: Boolean, @DrawableRes logo: Int, viewModel: LockViewModel = hiltViewModel()) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    if (!uiState.isUnlocked || !isContentReady) {
        LockScene(logo = logo)
    }
}
