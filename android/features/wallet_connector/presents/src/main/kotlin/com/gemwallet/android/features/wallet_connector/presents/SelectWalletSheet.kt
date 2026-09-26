@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.walletSections
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletSection

@Composable
internal fun SelectWalletSheet(isVisible: Boolean, walletSections: List<GemWalletSection>, selectedWalletId: WalletId?, onWalletSelected: (WalletId) -> Unit, onDismissRequest: () -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        dragHandle = { BottomSheetDefaults.DragHandle() },
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            item { SubheaderItem(R.string.wallets_title) }
            walletSections(walletSections, selectedWalletId?.id) { id ->
                onWalletSelected(WalletId(id))
                onDismissRequest()
            }
        }
    }
}
