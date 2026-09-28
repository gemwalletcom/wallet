package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ProviderRowView
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FiatProvidersList(isShow: MutableState<Boolean>, providers: List<GemProviderRow>, onProviderSelect: (FiatProviderName) -> Unit) {
    ModalBottomSheet(
        isVisible = isShow.value,
        onDismissRequest = { isShow.value = false },
        title = stringResource(R.string.buy_providers_title),
    ) {
        LazyColumn {
            itemsIndexed(providers) { index, row ->
                ProviderRowView(
                    row = row,
                    listPosition = ListPosition.getPosition(index, providers.size),
                    onClick = (row.kind as? GemProviderKind.Fiat)?.let { kind ->
                        {
                            onProviderSelect(kind.provider.toPrimitives())
                            isShow.value = false
                        }
                    },
                )
            }
        }
    }
}
