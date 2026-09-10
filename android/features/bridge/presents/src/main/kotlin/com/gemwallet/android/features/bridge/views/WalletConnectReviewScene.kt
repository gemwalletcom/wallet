package com.gemwallet.android.features.bridge.views

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectReviewModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AssetValueListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ext.networkName
import com.wallet.core.primitives.Chain
import uniffi.gemstone.MessageType

@Composable
internal fun WalletConnectReviewScene(
    model: WalletConnectReviewModel,
    buttonState: ButtonState,
    walletRow: @Composable (ListPosition) -> Unit,
    onApprove: () -> Unit,
    onReject: () -> Unit,
) {
    val context = LocalContext.current
    var sheetType by remember { mutableStateOf<WalletConnectReviewSheetType?>(null) }

    Scene(
        title = when (model.messageType) {
            MessageType.SIWE -> stringResource(R.string.common_sign_in_with, Chain.Ethereum.networkName())
            MessageType.SIWS -> stringResource(R.string.common_sign_in_with, Chain.Solana.networkName())
            MessageType.TEXT, MessageType.EIP712 -> stringResource(R.string.transfer_review_request)
        },
        backHandle = true,
        closeIcon = true,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.transfer_confirm),
                state = buttonState,
            ) {
                context.requestAuth(AuthRequest.Confirmation) {
                    onApprove()
                }
            }
        },
        onClose = onReject,
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            val header = model.header
            if (header == null) {
                item {
                    CenteredListHead(
                        icon = model.icon,
                        title = model.name,
                        subtitle = model.uri,
                        contentDescription = "wallet_connect_app_icon",
                        subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                    )
                }
                item { walletRow(ListPosition.First) }
            } else {
                item { AssetValueListHead(header) }
                item { PropertyItem(R.string.wallet_connect_app, model.name, listPosition = ListPosition.First) }
                item { walletRow(ListPosition.Middle) }
            }
            item {
                PropertyNetworkItem(model.chain, listPosition = ListPosition.Last)
            }
            simulationWarningsContent(model.warnings)
            if (model.hasPayload) {
                simulationPayloadFieldsContent(
                    fields = model.primaryPayloadFields,
                    addressNames = model.addressNames,
                    onDetailsClick = { sheetType = WalletConnectReviewSheetType.Details },
                )
            } else {
                walletConnectTextMessage(model.message)
            }
        }
    }

    WalletConnectPayloadDetailsSheet(
        isVisible = sheetType == WalletConnectReviewSheetType.Details,
        primaryFields = model.primaryPayloadFields,
        secondaryFields = model.secondaryPayloadFields,
        addressNames = model.addressNames,
        onViewFullMessage = { sheetType = WalletConnectReviewSheetType.FullMessage },
        onDismissRequest = { sheetType = null },
    )
    WalletConnectFullMessageSheet(
        isVisible = sheetType == WalletConnectReviewSheetType.FullMessage,
        message = model.message,
        onDismissRequest = { sheetType = null },
    )
}

private enum class WalletConnectReviewSheetType {
    Details,
    FullMessage,
}
