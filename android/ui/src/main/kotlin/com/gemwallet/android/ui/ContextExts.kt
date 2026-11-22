package com.gemwallet.android.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.view.Window
import android.view.WindowManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.application.security.cases.AuthRequester
import com.gemwallet.android.model.AuthRequest
import java.util.WeakHashMap

fun Context.findActivity(): Activity? {
    var context = this
    while (context is ContextWrapper) {
        if (context is Activity) return context
        context = context.baseContext
    }

    return null
}

fun Context.requestAuth(auth: AuthRequest, onCancel: () -> Unit = {}, onSuccess: () -> Unit) {
    (findActivity() as? AuthRequester)?.requestAuth(auth, onCancel, onSuccess)
}

fun Context.shareText(subject: String?, text: String?, chooserTitle: String) {
    val intent = Intent(Intent.ACTION_SEND)
    intent.type = "text/plain"
    intent.putExtra(Intent.EXTRA_SUBJECT, subject)
    intent.putExtra(Intent.EXTRA_TEXT, text)

    startActivity(Intent.createChooser(intent, chooserTitle))
}

private val secureScreens = WeakHashMap<Window, Int>()

@Composable
fun DisableScreenShooting() {
    val window = LocalContext.current.findActivity()?.window ?: return

    DisposableEffect(window) {
        secureScreens.merge(window, 1, Int::plus)
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)

        onDispose {
            if (secureScreens.computeIfPresent(window) { _, screens -> (screens - 1).takeIf { it > 0 } } == null) {
                window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
            }
        }
    }
}
