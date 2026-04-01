package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.getFiatProviderIcon
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.features.buy.viewmodels.models.BuyFiatProviderUIModel
import com.wallet.core.primitives.FiatProvider

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProviderList(
    isShow: MutableState<Boolean>,
    providers: List<BuyFiatProviderUIModel>,
    selectedProvider: BuyFiatProviderUIModel?,
    onProviderSelect: (FiatProvider) -> Unit,
) {
    if (!isShow.value) {
        return
    }

    ModalBottomSheet(
        onDismissRequest = { isShow.value = false },
    ) {
        LazyColumn {
            item {
                SubheaderItem(R.string.buy_providers_title)
            }
            itemsIndexed(providers) { index, item ->
                FiatProviderListItemView(
                    provider = item,
                    listPosition = ListPosition.getPosition(index, providers.size),
                    isSelected = item.provider.name == selectedProvider?.provider?.name,
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
        modifier = Modifier.clickable(onClick = onProviderSelect),
        leading = {
            AsyncImage(
                model = provider.provider.getFiatProviderIcon(),
                size = listItemIconSize,
            )
        },
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                ListItemTitleText(provider.provider.name)
                if (isSelected) {
                    Spacer8()
                    SelectionCheckmark()
                }
            }
        },
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(provider.cryptoText)
                ListItemSupportText(provider.fiatFormatted)
            }
        },
        listPosition = listPosition,
    )
}
