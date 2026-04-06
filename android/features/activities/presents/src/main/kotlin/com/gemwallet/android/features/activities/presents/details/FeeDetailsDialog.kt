package com.gemwallet.android.features.activities.presents.details

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.screen.ModalBottomSheet

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun FeeDetailsDialog(
    isVisible: Boolean,
    model: TransactionDetailsValue.Fee?,
    onCancel: () -> Unit,
) {
    model ?: return
    val sheetState = rememberModalBottomSheetState()
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onCancel,
        sheetState = sheetState,
    ) {
        model.asset.chain.asset().let {
            PropertyNetworkFee(
                networkTitle = it.name,
                networkSymbol = it.symbol,
                feeCrypto = model.value,
                feeFiat = model.equivalent,
                showedCryptoAmount = true
            )
        }
    }
}
