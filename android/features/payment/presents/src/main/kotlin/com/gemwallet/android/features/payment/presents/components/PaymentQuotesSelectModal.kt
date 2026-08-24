package com.gemwallet.android.features.payment.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.payment.viewmodels.model.PaymentQuoteUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
internal fun PaymentQuotesSelectModal(
    isVisible: Boolean,
    quotes: List<PaymentQuoteUIModel>,
    selected: String?,
    onSelect: (String) -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        title = stringResource(R.string.transfer_pay_with),
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            itemsIndexed(quotes) { index, quote ->
                ListItem(
                    modifier = Modifier.clickable { onSelect(quote.id) },
                    leading = {
                        IconWithBadge(
                            icon = quote.iconUrl,
                            placeholder = quote.symbol,
                            supportIcon = quote.supportIconUrl,
                        )
                    },
                    title = { ListItemTitleText(quote.name) },
                    subtitle = { ListItemSupportText(quote.networkName) },
                    trailing = {
                        getBalanceInfo(quote.amountText, quote.balance, false).invoke()
                        if (quote.id == selected) {
                            Spacer(modifier = Modifier.width(paddingSmall))
                            SelectionCheckmark()
                        }
                    },
                    listPosition = ListPosition.getPosition(index, quotes.size),
                )
            }
        }
    }
}
