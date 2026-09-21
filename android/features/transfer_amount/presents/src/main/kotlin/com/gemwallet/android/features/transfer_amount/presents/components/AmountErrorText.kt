package com.gemwallet.android.features.transfer_amount.presents.components

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.features.transfer_amount.presents.localization.text
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.infoSheet
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemAmountException

@Composable
fun amountErrorText(error: Throwable?): String = when (error) {
    null -> ""
    is GemAmountException -> error.display().text()
    else -> error.errorText().text()
}

@Composable
fun amountErrorInfo(error: Throwable?, onBuy: () -> Unit): InfoSheetEntity? {
    val context: Context = LocalContext.current
    return (error as? GemAmountException)?.display()?.info()?.infoSheet(context, null, onBuy)
}
