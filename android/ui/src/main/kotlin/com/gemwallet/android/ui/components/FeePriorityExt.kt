package com.gemwallet.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.FeePriority

@Composable
fun FeePriority.title(): String = when (this) {
    FeePriority.Normal -> stringResource(R.string.fee_rates_normal)
    FeePriority.Fast -> stringResource(R.string.fee_rates_fast)
}
