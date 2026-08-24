package com.gemwallet.android.features.payment.presents.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.payment.viewmodels.PaymentLinkError
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PaymentStatus
import uniffi.gemstone.PaymentException

@Composable
internal fun PaymentLinkError.message(): String = when (this) {
    PaymentLinkError.NoWallet,
    PaymentLinkError.QuoteUnavailable,
    PaymentLinkError.NoAccount -> stringResource(R.string.errors_not_supported)
    PaymentLinkError.WatchWallet -> stringResource(R.string.wallet_watch_tooltip_title)
    is PaymentLinkError.Gateway -> error.message()
}

@Composable
internal fun PaymentException?.message(): String = when (this) {
    is PaymentException.PaymentExpired,
    is PaymentException.QuoteExpired -> stringResource(R.string.errors_payment_expired)
    is PaymentException.Rejected -> stringResource(R.string.errors_payment_not_allowed)
    is PaymentException.PaymentNotFound -> stringResource(R.string.transaction_status_failed)
    is PaymentException.NoPaymentOptions,
    is PaymentException.NotSupported -> stringResource(R.string.errors_not_supported)
    is PaymentException.InvalidRequest,
    is PaymentException.Network,
    null -> stringResource(R.string.errors_error_occurred)
}

@Composable
internal fun PaymentStatus.message(): String? = when (this) {
    PaymentStatus.Succeeded -> stringResource(R.string.transaction_status_confirmed)
    PaymentStatus.Processing -> stringResource(R.string.transaction_status_pending)
    PaymentStatus.Cancelled -> null
    PaymentStatus.Expired -> stringResource(R.string.errors_payment_expired)
    PaymentStatus.Failed,
    PaymentStatus.RequiresAction -> stringResource(R.string.transaction_status_failed)
}
