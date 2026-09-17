package com.gemwallet.android.ui.components.screen

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import com.gemwallet.android.ui.models.ToastMessage
import kotlinx.coroutines.flow.Flow

@Composable
fun ToastEffect(
    events: Flow<ToastMessage>,
    snackbar: SnackbarHostState,
) {
    LaunchedEffect(events, snackbar) {
        events.collect { event -> snackbar.showSnackbar(event.title, event.image) }
    }
}
