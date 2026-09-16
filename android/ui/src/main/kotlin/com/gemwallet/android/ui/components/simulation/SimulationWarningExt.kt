package com.gemwallet.android.ui.components.simulation

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.SimulationSeverity

@Composable
fun SimulationSeverity.color(): Color = when (this) {
    SimulationSeverity.CRITICAL -> MaterialTheme.colorScheme.error
    else -> pendingColor
}
