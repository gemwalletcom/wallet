package com.gemwallet.android.features.activities.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R

@StringRes
internal fun TransactionDetailsValue.Destination.stringRes(): Int = when (this) {
    is TransactionDetailsValue.Destination.Recipient -> R.string.transaction_recipient
    is TransactionDetailsValue.Destination.Sender -> R.string.transaction_sender
    is TransactionDetailsValue.Destination.Contract -> R.string.asset_contract
    is TransactionDetailsValue.Destination.Validator -> R.string.stake_validator
    is TransactionDetailsValue.Destination.ProviderAddress -> R.string.common_provider
    is TransactionDetailsValue.Destination.Provider -> R.string.common_provider
}
