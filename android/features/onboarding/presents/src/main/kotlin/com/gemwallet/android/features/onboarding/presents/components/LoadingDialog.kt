package com.gemwallet.android.features.onboarding.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.gemwallet.android.ui.theme.paddingSmall

private val loadingDialogSize = 100.dp

@Composable
internal fun LoadingDialog() {
    Dialog(
        onDismissRequest = {},
        DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false),
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .size(loadingDialogSize)
                .background(
                    MaterialTheme.colorScheme.background,
                    shape = RoundedCornerShape(paddingSmall),
                ),
        ) {
            CircularProgressIndicator()
        }
    }
}
