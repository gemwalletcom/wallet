package com.gemwallet.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import uniffi.gemstone.FeeOption

@Composable
fun FeeOption.title(): String = when (this) {
    FeeOption.TOKEN_ACCOUNT_CREATION -> stringResource(R.string.banner_account_activation_title)
}
