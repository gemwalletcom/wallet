package com.gemwallet.android.ui.models

import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning

fun List<SimulationWarning>.hasCriticalWarning(): Boolean =
    any { it.severity == SimulationSeverity.Critical }
