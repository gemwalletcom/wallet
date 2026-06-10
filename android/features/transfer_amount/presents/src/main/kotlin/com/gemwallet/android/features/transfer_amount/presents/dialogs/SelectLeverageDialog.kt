package com.gemwallet.android.features.transfer_amount.presents.dialogs

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SelectLeverageDialog(
    isVisible: Boolean,
    leverages: List<Int>,
    selected: Int,
    onDismiss: () -> Unit,
    onSelect: (Int) -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.perpetual_leverage),
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxWidth()
        ) {
            itemsPositioned(leverages) { position, item ->
                ListItem(
                    modifier = Modifier.clickable {
                        onSelect(item)
                        onDismiss()
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                    title = { ListItemTitleText(item.formatLeverage()) },
                    listPosition = position,
                    trailing = if (item == selected) {
                        @Composable { SelectionCheckmark() }
                    } else {
                        null
                    },
                )
            }
        }
    }
}
