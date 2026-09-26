package com.gemwallet.android.features.transactions.presents.transaction

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemTransactionFeeRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun FeeDetailsDialog(isVisible: Boolean, feeRow: GemTransactionFeeRow, onCancel: () -> Unit) {
    val context = LocalContext.current
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        title = stringResource(R.string.transfer_network_fee),
    ) {
        ListItem(
            model = ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.fee.amount.text(),
                subtitleExtra = feeRow.fee.fiat?.text(),
            ),
            listPosition = ListPosition.Single,
        )
    }
}
