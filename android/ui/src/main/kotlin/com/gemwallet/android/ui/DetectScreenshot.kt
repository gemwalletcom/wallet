package com.gemwallet.android.ui

import android.app.Activity
import android.os.Build
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource

@Composable
fun DetectScreenshot(docsUrl: String) {
    val context = LocalContext.current
    var showWarning by remember { mutableStateOf(false) }

    DisposableEffect(context) {
        val activity = context.findActivity()
        if (activity != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val callback = Activity.ScreenCaptureCallback { showWarning = true }
            activity.registerScreenCaptureCallback(activity.mainExecutor, callback)
            onDispose { activity.unregisterScreenCaptureCallback(callback) }
        } else {
            onDispose { }
        }
    }

    if (showWarning) {
        ScreenshotDetectedDialog(
            docsUrl = docsUrl,
            onDismiss = { showWarning = false },
        )
    }
}

@Composable
private fun ScreenshotDetectedDialog(docsUrl: String, onDismiss: () -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(text = stringResource(R.string.secret_phrase_screenshot_detected_title)) },
        text = { Text(text = stringResource(R.string.secret_phrase_screenshot_detected_description)) },
        confirmButton = {
            Button(onClick = onDismiss) {
                Text(text = stringResource(R.string.common_done))
            }
        },
        dismissButton = {
            TextButton(
                onClick = {
                    uriHandler.open(context, docsUrl)
                    onDismiss()
                }
            ) {
                Text(text = stringResource(R.string.common_learn_more))
            }
        },
    )
}
