package com.gemwallet.android.features.payment.presents

internal sealed interface PaymentSceneAction {
    data class SelectQuote(val quoteId: String) : PaymentSceneAction
    data object ConfirmQuote : PaymentSceneAction
    data object DataCollected : PaymentSceneAction
    data class DataCollectionFailed(val message: String?) : PaymentSceneAction
    data object DismissDataCollection : PaymentSceneAction
    data object BackFromConfirm : PaymentSceneAction
    data class TransactionHash(val hash: String) : PaymentSceneAction
    data object Retry : PaymentSceneAction
    data object Cancel : PaymentSceneAction
}
