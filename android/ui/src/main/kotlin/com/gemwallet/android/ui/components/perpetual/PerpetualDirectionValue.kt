package com.gemwallet.android.ui.components.perpetual

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.PerpetualDirection

@Composable
fun PerpetualDirection.title(): String = stringResource(stringRes())

@Composable
fun PerpetualDirection.titleAndLeverage(leverage: Int): String = "${title()} ${leverage}x"

@Composable
fun PerpetualDirection.color(): Color = when (this) {
    PerpetualDirection.Short -> MaterialTheme.colorScheme.error
    PerpetualDirection.Long -> MaterialTheme.colorScheme.tertiary
}
