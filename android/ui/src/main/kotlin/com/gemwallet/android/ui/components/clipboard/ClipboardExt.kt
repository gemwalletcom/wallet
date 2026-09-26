package com.gemwallet.android.ui.components.clipboard

import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.PersistableBundle
import android.widget.Toast
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemCopy
import java.util.UUID

fun ClipboardManager.setPlainText(context: Context, data: String) = setClip(context, data, isSensitive = false, expirySeconds = null)

fun ClipboardManager.setCopy(context: Context, copy: GemCopy) = setClip(context, copy.value, isSensitive = copy.kind.isSensitive(), expirySeconds = copy.kind.clipboardExpirySeconds())

private fun ClipboardManager.setClip(context: Context, data: String, isSensitive: Boolean, expirySeconds: UInt?) {
    val label = expirySeconds?.let { UUID.randomUUID().toString() }.orEmpty()
    val clip = ClipData.newPlainText(label, data).apply {
        if (isSensitive) {
            description.extras = PersistableBundle().apply {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    putBoolean(ClipDescription.EXTRA_IS_SENSITIVE, true)
                } else {
                    putBoolean("android.content.extra.IS_SENSITIVE", true)
                }
            }
        }
    }
    setPrimaryClip(clip)

    expirySeconds?.let { seconds ->
        Handler(Looper.getMainLooper()).postDelayed({ clearExpiredClip(label) }, seconds.toLong() * 1000)
    }

    if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.S_V2) {
        Toast.makeText(context, context.getString(R.string.common_copied_to_clipboard), Toast.LENGTH_SHORT).show()
    }
}

private fun ClipboardManager.clearExpiredClip(label: String) {
    val current = primaryClipDescription
    if (current == null || current.label == label) {
        clearPrimaryClip()
    }
}

fun ClipboardManager.getPlainText(): String? = primaryClip?.getItemAt(0)?.text?.toString()

fun ClipboardManager.clear() {
    clearPrimaryClip()
}

fun Context.clipboardManager(): ClipboardManager = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
