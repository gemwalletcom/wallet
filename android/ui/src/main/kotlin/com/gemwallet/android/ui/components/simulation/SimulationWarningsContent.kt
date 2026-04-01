package com.gemwallet.android.ui.components.simulation

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.SimulationWarning

fun LazyListScope.simulationWarningsContent(warnings: List<SimulationWarning>) {
    if (warnings.isEmpty()) return

    itemsIndexed(warnings) { index, warning ->
        WarningItem(
            title = stringResource(warning.titleRes()),
            message = warning.descriptionText(),
            color = warning.severity.color(),
            position = ListPosition.getPosition(index, warnings.size),
        )
    }
}
