package com.gemwallet.android.ui.components.clipboard

import android.content.ClipboardManager
import android.content.Context
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemCopyKind

fun ClipboardManager.setCopy(context: Context, copy: GemCopy) = setPlainText(
    context = context,
    data = copy.value,
    isSensitive = when (copy.kind) {
        is GemCopyKind.Address -> false
        GemCopyKind.SecretPhrase, GemCopyKind.PrivateKey -> true
    },
)
