package com.gemwallet.android.ui.components.perpetual

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualType

@Composable
fun PerpetualType.title(): String {
    val (direction, formatRes) = when (this) {
        is PerpetualType.Open -> content.direction to null
        is PerpetualType.Close -> content.direction to R.string.perpetual_close_direction
        is PerpetualType.Increase -> content.direction to R.string.perpetual_increase_direction
        is PerpetualType.Reduce -> content.positionDirection to R.string.perpetual_reduce_direction
        is PerpetualType.Modify -> return stringResource(R.string.perpetual_modify)
    }
    val directionLabel = stringResource(when (direction) {
        PerpetualDirection.Long -> R.string.perpetual_long
        PerpetualDirection.Short -> R.string.perpetual_short
    })
    return formatRes?.let { stringResource(it, directionLabel) } ?: directionLabel
}
