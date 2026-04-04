package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.sceneContentPaddingValues

@Composable
fun LoadingScene(
    title: String,
    onCancel: () -> Unit,
    closeIcon: Boolean = false,
) {
    Scene(
        title = title,
        padding = sceneContentPaddingValues(),
        onClose = onCancel,
        closeIcon = closeIcon,
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
        }
    }
}
