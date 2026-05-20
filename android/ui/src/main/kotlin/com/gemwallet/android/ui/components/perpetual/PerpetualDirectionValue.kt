package com.gemwallet.android.ui.components.perpetual

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PerpetualDirection

@Composable
fun PerpetualDirection.text(leverage: Int): String = when (this) {
    PerpetualDirection.Short -> "${stringResource(R.string.perpetual_short).uppercase()} ${leverage}x"
    PerpetualDirection.Long -> "${stringResource(R.string.perpetual_long).uppercase()} ${leverage}x"
}

@Composable
fun PerpetualDirection.title(): String = when (this) {
    PerpetualDirection.Long -> stringResource(R.string.perpetual_long)
    PerpetualDirection.Short -> stringResource(R.string.perpetual_short)
}

@Composable
fun PerpetualDirection.titleAndLeverage(leverage: Int): String = "${title()} ${leverage}x"

@Composable
fun PerpetualDirection.color(): Color = when (this) {
    PerpetualDirection.Short -> MaterialTheme.colorScheme.error
    PerpetualDirection.Long -> MaterialTheme.colorScheme.tertiary
}
