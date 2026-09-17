package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.buy.viewmodels.models.BuyFiatProviderUIModel
import com.gemwallet.android.features.buy.viewmodels.models.listItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.FiatProviderName

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProviderList(
    isShow: MutableState<Boolean>,
    providers: List<BuyFiatProviderUIModel>,
    selectedProvider: BuyFiatProviderUIModel?,
    onProviderSelect: (FiatProviderName) -> Unit,
) {
    ModalBottomSheet(
        isVisible = isShow.value,
        onDismissRequest = { isShow.value = false },
        title = stringResource(R.string.buy_providers_title),
    ) {
        LazyColumn {
            itemsIndexed(providers) { index, item ->
                FiatProviderListItemView(
                    provider = item,
                    listPosition = ListPosition.getPosition(index, providers.size),
                    isSelected = item.provider == selectedProvider?.provider,
                    onProviderSelect = {
                        onProviderSelect(item.provider)
                        isShow.value = false
                    },
                )
            }
        }
    }
}

@Composable
private fun FiatProviderListItemView(
    provider: BuyFiatProviderUIModel,
    listPosition: ListPosition,
    isSelected: Boolean,
    onProviderSelect: () -> Unit,
) {
    ListItem(
        model = provider.listItem(),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onProviderSelect),
        accessory = if (isSelected) {
            { SelectionCheckmark() }
        } else {
            null
        },
    )
}
