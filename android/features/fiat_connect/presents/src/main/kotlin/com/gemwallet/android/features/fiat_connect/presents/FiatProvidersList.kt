package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.fiat_connect.viewmodels.models.FiatQuoteUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import com.wallet.core.primitives.FiatProviderName

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FiatProvidersList(isShow: MutableState<Boolean>, providers: List<FiatQuoteUIModel>, selectedProvider: FiatQuoteUIModel?, onProviderSelect: (FiatProviderName) -> Unit) {
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
private fun FiatProviderListItemView(provider: FiatQuoteUIModel, listPosition: ListPosition, isSelected: Boolean, onProviderSelect: () -> Unit) {
    ListItem(
        modifier = Modifier.clickable(onClick = onProviderSelect),
        listPosition = listPosition,
        leading = {
            if (isSelected) {
                IconWithBadge(
                    icon = provider.provider.iconModel(),
                    size = listItemIconSize,
                    badge = { SelectionCheckmark() },
                )
            } else {
                AsyncImage(
                    model = provider.provider.iconModel(),
                    size = listItemIconSize,
                )
            }
        },
        title = { ListItemTitleText(provider.providerName) },
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(provider.cryptoText)
                ListItemSupportText(provider.fiatFormatted)
            }
        },
    )
}
