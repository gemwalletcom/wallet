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
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun WalletConnectReviewScene(
    model: WalletConnectReviewModel,
    buttonState: ButtonState,
    walletRow: @Composable () -> Unit,
    onApprove: () -> Unit,
    onReject: () -> Unit,
) {
    val context = LocalContext.current
    var sheetType by remember { mutableStateOf<WalletConnectReviewSheetType?>(null) }

    Scene(
        title = stringResource(id = R.string.transfer_review_request),
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
            item {
                CenteredListHead(
                    icon = model.icon,
                    title = model.name,
                    subtitle = model.uri,
                    contentDescription = "wallet_connect_app_icon",
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item { walletRow() }
            item {
                PropertyNetworkItem(model.chain, listPosition = ListPosition.Last)
            }
            simulationWarningsContent(model.warnings)
            if (model.hasPayload) {
                simulationPayloadFieldsContent(
                    fields = model.primaryPayloadFields,
                    onDetailsClick = { sheetType = WalletConnectReviewSheetType.Details },
                )
            } else {
                walletConnectTextMessage(model.message)
            }
        }
    }

    when (sheetType) {
        WalletConnectReviewSheetType.Details -> {
            WalletConnectPayloadDetailsSheet(
                primaryFields = model.primaryPayloadFields,
                secondaryFields = model.secondaryPayloadFields,
                onViewFullMessage = { sheetType = WalletConnectReviewSheetType.FullMessage },
                onDismissRequest = { sheetType = null },
            )
        }
        WalletConnectReviewSheetType.FullMessage -> {
            WalletConnectFullMessageSheet(
                message = model.message,
                onDismissRequest = { sheetType = null },
            )
        }
        null -> Unit
    }
}

private enum class WalletConnectReviewSheetType {
    Details,
    FullMessage,
}
