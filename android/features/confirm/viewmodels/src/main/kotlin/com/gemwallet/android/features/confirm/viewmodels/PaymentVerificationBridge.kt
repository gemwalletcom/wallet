package com.gemwallet.android.features.confirm.viewmodels

import android.webkit.JavascriptInterface
import com.gemwallet.android.ui.components.WebViewBridge
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import uniffi.gemstone.GemPaymentVerificationOutcome
import uniffi.gemstone.paymentVerificationOutcome

class PaymentVerificationBridge(private val onComplete: () -> Unit, private val onError: () -> Unit) : WebViewBridge {

    override val name: String = "AndroidWallet"

    @JavascriptInterface
    fun onDataCollectionComplete(message: String) {
        val type = runCatching { Json.parseToJsonElement(message).jsonObject[MESSAGE_TYPE]?.jsonPrimitive?.content }.getOrNull() ?: return
        when (paymentVerificationOutcome(type)) {
            GemPaymentVerificationOutcome.COMPLETE -> onComplete()
            GemPaymentVerificationOutcome.ERROR -> onError()
            GemPaymentVerificationOutcome.IGNORED -> Unit
        }
    }

    private companion object {
        const val MESSAGE_TYPE = "type"
    }
}
