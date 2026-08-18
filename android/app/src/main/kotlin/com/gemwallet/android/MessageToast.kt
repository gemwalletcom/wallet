package com.gemwallet.android

import android.widget.Toast
import android.widget.Toast.makeText
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource

@Composable
internal fun MessageToast(
    visible: Boolean,
    @StringRes message: Int,
    onShown: () -> Unit,
) {
    val context = LocalContext.current
    val text = stringResource(id = message)
    LaunchedEffect(visible) {
        if (visible) {
            makeText(context, text, Toast.LENGTH_SHORT).show()
            onShown()
        }
    }
}
