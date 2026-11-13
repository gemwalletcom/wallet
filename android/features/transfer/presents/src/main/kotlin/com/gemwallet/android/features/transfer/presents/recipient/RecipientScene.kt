package com.gemwallet.android.features.transfer.presents.recipient

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.transfer.presents.components.ContinueScene
import com.gemwallet.android.features.transfer.presents.recipient.components.RecipientHead
import com.gemwallet.android.features.transfer.presents.recipient.components.destinationView
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.QrScanField
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientHeadUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.gemGroups
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemGroup
import uniffi.gemstone.GemListItem
import uniffi.gemstone.GemNameIndicator

@Composable
internal fun RecipientScene(
    asset: Asset,
    head: RecipientHeadUIModel,
    hasMemo: Boolean,
    address: String,
    memo: String,
    addressError: String,
    nameResolveIndicator: GemNameIndicator?,
    sections: List<GemGroup>,
    buttonState: ButtonState,
    onAction: (RecipientAction) -> Unit,
) {
    ContinueScene(
        title = stringResource(id = R.string.transfer_recipient_title),
        buttonState = buttonState,
        onClose = { onAction(RecipientAction.Cancel) },
        onContinue = { onAction(RecipientAction.Next) },
    ) {
        val onAddress: (String) -> Unit = { onAction(RecipientAction.SetAddress(it)) }
        val onMemo: (String) -> Unit = { onAction(RecipientAction.SetMemo(it)) }
        val onQrScan: (QrScanField) -> Unit = { onAction(RecipientAction.Scan(it)) }
        val onSubmitAddress: () -> Unit = { onAction(RecipientAction.ValidateAddress) }
        LazyColumn(
            contentPadding = PaddingValues(bottom = paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            item { RecipientHead(head) }
            destinationView(
                hasMemo = hasMemo,
                address = address,
                addressError = addressError,
                nameResolveIndicator = nameResolveIndicator,
                memo = memo,
                onAddress = onAddress,
                onMemo = onMemo,
                onQrScan = onQrScan,
                onSubmitAddress = onSubmitAddress,
            )
            gemGroups(sections) { item, position ->
                if (item is GemListItem.Recipient) {
                    ListItem(
                        model = ListItemModel(title = item.row.title, subtitle = item.row.subtitle),
                        listPosition = position,
                        modifier = Modifier.clickable {
                            item.row.recipient.memo?.let { onAction(RecipientAction.SetMemo(it)) }
                            onAction(RecipientAction.Select(item.row.recipient))
                        }.testTag("recipient_row"),
                        accessory = { DataBadgeChevron() },
                    )
                }
            }
        }
    }
}
