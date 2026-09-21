package com.gemwallet.android.ui.components.perpetual

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemPerpetualConfirmDetails

@Composable
fun PerpetualDetailsSummaryItem(details: GemPerpetualConfirmDetails, onClick: () -> Unit, listPosition: ListPosition = ListPosition.Single) {
    ListItem(
        model = ListItemModel(
            title = stringResource(R.string.common_details),
            subtitle = details.summary.text?.string(LocalContext.current).orEmpty(),
            subtitleStyle = details.summary.tone.textStyle(),
        ),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        accessory = { DataBadgeChevron() },
    )
}

@Composable
fun PerpetualDetailsBottomSheet(isVisible: Boolean, details: GemPerpetualConfirmDetails?, onDismiss: () -> Unit) {
    ModalBottomSheet(
        item = details.takeIf { isVisible },
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = { stringResource(R.string.common_details) },
    ) { details ->
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            gemListSections(details.sections)
        }
    }
}
