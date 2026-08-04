@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.bridge.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

@Composable
internal fun WalletSelectionSheet(
    isVisible: Boolean,
    wallets: List<Wallet>,
    selectedWalletId: WalletId?,
    onWalletSelected: (WalletId) -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        dragHandle = { BottomSheetDefaults.DragHandle() },
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            item { SubheaderItem(R.string.wallets_title) }
            itemsIndexed(wallets) { index, wallet ->
                WalletItem(
                    wallet = wallet,
                    isCurrent = wallet.id == selectedWalletId,
                    listPosition = ListPosition.getPosition(index, wallets.size),
                    modifier = Modifier.clickable {
                        onWalletSelected(wallet.id)
                        onDismissRequest()
                    },
                )
            }
        }
    }
}
