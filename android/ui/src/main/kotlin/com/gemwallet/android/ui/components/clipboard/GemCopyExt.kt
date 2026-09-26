package com.gemwallet.android.ui.components.clipboard

import android.content.ClipboardManager
import android.content.Context
import uniffi.gemstone.GemCopy

fun ClipboardManager.setCopy(context: Context, copy: GemCopy) = setClip(context, copy.value, isSensitive = copy.kind.isSensitive(), expirySeconds = copy.kind.clipboardExpirySeconds())
