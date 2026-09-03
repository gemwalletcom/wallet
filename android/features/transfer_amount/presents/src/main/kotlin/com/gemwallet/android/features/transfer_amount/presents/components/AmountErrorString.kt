package com.gemwallet.android.features.transfer_amount.presents.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.features.transfer_amount.models.AmountError

@Composable
fun amountErrorString(error: AmountError): String = when (error) {
    AmountError.None -> ""
    AmountError.IncorrectAmount -> stringResource(id = R.string.errors_invalid_amount)
    AmountError.Required -> stringResource(
        id = R.string.common_required_field,
        stringResource(id = R.string.transfer_amount)
    )
    is AmountError.InsufficientBalance -> stringResource(
        id = R.string.transfer_insufficient_balance,
        error.assetSymbol
    )
    is AmountError.MinimumValue -> stringResource(
        id = R.string.transfer_minimum_amount,
        error.minimumValue
    )
    is AmountError.Unknown -> error.data.takeIf { it.isNotBlank() }
        ?.let { "${stringResource(id = R.string.errors_unknown)}: $it" }
        ?: stringResource(id = R.string.errors_unknown_try_again)
    AmountError.NoValidatorSelected -> stringResource(id = R.string.errors_unknown)
    AmountError.NoDelegationSelected -> stringResource(id = R.string.errors_unknown)
}