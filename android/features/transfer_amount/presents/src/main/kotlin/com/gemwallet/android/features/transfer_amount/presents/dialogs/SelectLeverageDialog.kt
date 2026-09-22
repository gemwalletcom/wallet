package com.gemwallet.android.features.transfer_amount.presents.dialogs

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.localization.string
import uniffi.gemstone.GemPickerOption

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SelectLeverageDialog(isVisible: Boolean, leverages: List<GemPickerOption>, selected: GemPickerOption?, onDismiss: () -> Unit, onSelect: (UByte) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.perpetual_leverage),
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxWidth(),
        ) {
            itemsPositioned(leverages) { position, item ->
                ListItem(
                    model = ListItemModel(title = item.label.string(LocalContext.current)),
                    listPosition = position,
                    modifier = Modifier.clickable {
                        onSelect(item.value)
                        onDismiss()
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                    accessory = if (item == selected) {
                        @Composable { SelectionCheckmark() }
                    } else {
                        null
                    },
                )
            }
        }
    }
}
