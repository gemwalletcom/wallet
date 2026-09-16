package com.gemwallet.android.features.transfer_amount.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.features.transfer_amount.presents.localization.text
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemAmountException

@Composable
fun amountErrorText(error: Throwable?): String = when (error) {
    null -> ""
    is GemAmountException -> error.display().text()
    else -> error.errorText().text()
}
