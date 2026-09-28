package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemSimulationPayloadRow

@Composable
internal fun SignMessagePayloadDetailsSheet(
    isVisible: Boolean,
    primaryFields: List<GemSimulationPayloadRow>,
    secondaryFields: List<GemSimulationPayloadRow>,
    onAddressClick: (String) -> Unit,
    onViewFullMessage: () -> Unit,
    onDismissRequest: () -> Unit,
    viewFullMessageListItem: ListItemModel,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        expansion = SheetExpansion.Full,
        onDismissRequest = onDismissRequest,
        title = stringResource(R.string.common_details),
    ) {
        LazyColumn {
            simulationPayloadDetailsContent(
                primaryFields = primaryFields,
                secondaryFields = secondaryFields,
                onAddressClick = onAddressClick,
            )
            item {
                ListItem(
                    model = viewFullMessageListItem,
                    listPosition = ListPosition.Single,
                    modifier = Modifier.clickable(onClick = onViewFullMessage),
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
