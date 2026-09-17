package com.gemwallet.android.features.confirm.viewmodels

import android.webkit.JavascriptInterface
import com.gemwallet.android.ui.components.WebViewBridge
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

class PaymentVerificationBridge(private val onComplete: () -> Unit) : WebViewBridge {

    override val name: String = "AndroidWallet"

    @JavascriptInterface
    fun onDataCollectionComplete(message: String) {
        val type = runCatching { Json.parseToJsonElement(message).jsonObject[MESSAGE_TYPE]?.jsonPrimitive?.content }.getOrNull()
        if (type == COMPLETE_TYPE) {
            onComplete()
        }
    }

    private companion object {
        const val MESSAGE_TYPE = "type"
        const val COMPLETE_TYPE = "IC_COMPLETE"
    }
}
