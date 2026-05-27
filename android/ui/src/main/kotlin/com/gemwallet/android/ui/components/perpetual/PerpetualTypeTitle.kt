package com.gemwallet.android.ui.components.perpetual

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualType

@Composable
fun PerpetualType.title(): String = when (this) {
    is PerpetualType.Open -> directionLabel(content.direction)
    is PerpetualType.Increase -> stringResource(R.string.perpetual_increase_direction, directionLabel(content.direction))
    is PerpetualType.Reduce -> stringResource(R.string.perpetual_reduce_direction, directionLabel(content.positionDirection))
    is PerpetualType.Close -> stringResource(R.string.perpetual_close_position)
    is PerpetualType.Modify -> stringResource(R.string.perpetual_modify_position)
}

@Composable
private fun directionLabel(direction: PerpetualDirection): String = stringResource(when (direction) {
    PerpetualDirection.Long -> R.string.perpetual_long
    PerpetualDirection.Short -> R.string.perpetual_short
})
