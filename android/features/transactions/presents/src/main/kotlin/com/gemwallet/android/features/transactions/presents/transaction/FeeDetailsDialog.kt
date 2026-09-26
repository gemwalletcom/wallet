package com.gemwallet.android.features.transactions.presents.transaction

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun FeeDetailsDialog(isVisible: Boolean, model: ListItemModel?, onCancel: () -> Unit) {
    model ?: return
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        title = stringResource(R.string.transfer_network_fee),
    ) {
        ListItem(model = model, listPosition = ListPosition.Single)
    }
}
