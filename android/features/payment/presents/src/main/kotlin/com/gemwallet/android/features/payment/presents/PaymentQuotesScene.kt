package com.gemwallet.android.features.payment.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.gemwallet.android.features.payment.presents.components.PaymentDataCollectionModal
import com.gemwallet.android.features.payment.presents.components.PaymentQuotesSelectModal
import com.gemwallet.android.features.payment.viewmodels.PaymentSceneState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.IconPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer8

@Composable
internal fun PaymentQuotesScene(
    state: PaymentSceneState.Quotes,
    onAction: (PaymentSceneAction) -> Unit,
) {
    var isSelectingQuote by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        backHandle = true,
        onClose = { onAction(PaymentSceneAction.Cancel) },
        mainAction = {
            MainActionButton(
                state = if (state.selected == null) ButtonState.Disabled else ButtonState.Enabled,
                onClick = { onAction(PaymentSceneAction.ConfirmQuote) },
            ) {
                if (state.selectedQuote?.requiresVerification == true) {
                    Icon(AppIcons.Person, contentDescription = null)
                    Spacer8()
                }
                Text(
                    text = stringResource(R.string.common_continue),
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Medium,
                )
            }
        },
    ) {
        LazyColumn {
            item {
                CenteredListHead(
                    icon = state.selectedQuote?.iconUrl ?: state.merchant.iconUrl,
                    title = state.selectedQuote?.amountText ?: state.merchant.name,
                    subtitle = state.price,
                    placeholderText = state.merchant.name.firstOrNull()?.uppercaseChar()?.toString(),
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item {
                IconPropertyItem(
                    title = R.string.transfer_recipient_title,
                    text = state.merchant.name,
                    icon = state.merchant.iconUrl,
                    listPosition = ListPosition.First,
                )
            }
            item {
                IconPropertyItem(
                    title = R.string.common_wallet,
                    text = state.walletName,
                    icon = walletItemIconModel(state.walletType, state.walletChain),
                    listPosition = ListPosition.Last,
                )
            }
            item {
                val isSelectable = state.quotes.size > 1
                PropertyItem(
                    modifier = Modifier.clickable(enabled = isSelectable) { isSelectingQuote = true },
                    title = { PropertyTitleText(R.string.transfer_pay_with) },
                    data = {
                        PropertyDataText(
                            text = state.selectedQuote?.amountText.orEmpty(),
                            badge = { DataBadgeChevron(icon = state.selectedQuote?.iconUrl.orEmpty(), isShowChevron = isSelectable) },
                        )
                    },
                    listPosition = ListPosition.Single,
                )
            }
        }
    }

    PaymentDataCollectionModal(
        url = state.collectData,
        onAction = onAction,
    )

    PaymentQuotesSelectModal(
        isVisible = isSelectingQuote,
        quotes = state.quotes,
        selected = state.selected,
        onSelect = {
            onAction(PaymentSceneAction.SelectQuote(it))
            isSelectingQuote = false
        },
        onDismissRequest = { isSelectingQuote = false },
    )
}
