package com.gemwallet.android.features.swap.views.dialogs

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.dialog.DialogBar
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.swap.SwapProviderListItemView
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import com.gemwallet.android.ui.theme.defaultPadding
import uniffi.gemstone.SwapperProvider

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun SwapProviderListDialog(
    isVisible: Boolean,
    isUpdated: Boolean,
    currentProvider: SwapperProvider?,
    providers: List<SwapProviderUIModel>,
    onDismiss: () -> Unit,
    onProviderSelect: (SwapperProvider) -> Unit,
) {
    if (!isVisible) {
        return
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        dragHandle = {
            DialogBar(onDismissRequest = onDismiss, showDismissAction = false)
        }
    ) {
        if (isUpdated) {
            Box(modifier = Modifier.fillMaxWidth().defaultPadding()) {
                CircularProgressIndicator20(modifier = Modifier.align(Alignment.Center))
            }
            return@ModalBottomSheet
        }
        LazyColumn {
            itemsIndexed(providers) { index, item ->
                SwapProviderListItemView(
                    provider = item,
                    listPosition = ListPosition.getPosition(index, providers.size),
                    isSelected = item.id == currentProvider,
                    onProviderSelect = {
                        onDismiss()
                        onProviderSelect(it)
                    }
                )
            }
        }
    }
}
