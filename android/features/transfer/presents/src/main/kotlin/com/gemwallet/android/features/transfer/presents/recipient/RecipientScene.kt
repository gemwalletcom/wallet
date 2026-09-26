package com.gemwallet.android.features.transfer.presents.recipient

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.transfer.presents.recipient.components.RecipientHead
import com.gemwallet.android.features.transfer.presents.recipient.components.destinationView
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientHeadUIModel
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset

@Composable
internal fun RecipientScene(
    asset: Asset,
    head: RecipientHeadUIModel,
    hasMemo: Boolean,
    address: String,
    memo: String,
    addressError: String,
    nameResolveIndicator: NameResolveIndicatorUIModel?,
    sections: List<ListSection<RecipientRowUIModel>>,
    buttonState: ButtonState,
    onAction: (RecipientAction) -> Unit,
) {
    val isKeyBoardOpen = WindowInsets.isKeyboardVisible
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < SceneSizing.compactContentHeight
    }

    Scene(
        title = stringResource(id = R.string.transfer_recipient_title),
        onClose = { onAction(RecipientAction.Cancel) },
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    state = buttonState,
                    onClick = { onAction(RecipientAction.Next) },
                )
            }
        },
        actions = {
            TextButton(
                onClick = { onAction(RecipientAction.Next) },
                enabled = buttonState == ButtonState.Enabled,
                colors = ButtonDefaults.textButtonColors()
                    .copy(contentColor = MaterialTheme.colorScheme.primary),
            ) {
                Text(stringResource(R.string.common_continue).uppercase())
            }
        },
    ) {
        LazyColumn(
            contentPadding = PaddingValues(bottom = paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            item { RecipientHead(asset, head) }
            destinationView(
                hasMemo = hasMemo,
                address = address,
                addressError = addressError,
                nameResolveIndicator = nameResolveIndicator,
                memo = memo,
                onAddress = { onAction(RecipientAction.SetAddress(it)) },
                onMemo = { onAction(RecipientAction.SetMemo(it)) },
                onQrScan = { onAction(RecipientAction.Scan(it)) },
                onSubmitAddress = { onAction(RecipientAction.ValidateAddress) },
            )
            listSections(sections) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        item.recipient.memo?.let { onAction(RecipientAction.SetMemo(it)) }
                        onAction(RecipientAction.Select(item.recipient))
                    },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
