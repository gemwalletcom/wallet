package com.gemwallet.android.features.assets.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyStateView

@Composable
fun WalletDefiSection() {
    EmptyStateView(
        title = stringResource(R.string.earn_state_empty_title),
        icon = painterResource(R.drawable.empty_activity),
    )
}
