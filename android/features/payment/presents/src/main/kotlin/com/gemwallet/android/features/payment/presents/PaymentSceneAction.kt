package com.gemwallet.android.features.payment.presents

internal sealed interface PaymentSceneAction {
    data class SelectQuote(val quoteId: String) : PaymentSceneAction
    data object ConfirmQuote : PaymentSceneAction
    data object DataCollected : PaymentSceneAction
    data class DataCollectionFailed(val message: String?) : PaymentSceneAction
    data object Sign : PaymentSceneAction
    data class ActionResult(val result: String) : PaymentSceneAction
    data object Cancel : PaymentSceneAction
}
