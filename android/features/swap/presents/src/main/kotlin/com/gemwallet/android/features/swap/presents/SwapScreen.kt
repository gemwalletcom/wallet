package com.gemwallet.android.features.swap.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.swap.presents.dialogs.PriceImpactWarningDialog
import com.gemwallet.android.features.swap.viewmodels.SwapViewModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.ObserveStartedState
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapSlippageBottomSheet
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.AssetId

@Composable
fun SwapScreen(
    select: SwapItemType?,
    selectedAssetId: AssetId?,
    viewModel: SwapViewModel = hiltViewModel(),
    onSelectionConsumed: () -> Unit,
    onSelect: (select: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) -> Unit,
    onConfirm: (ConfirmTransferInput) -> Unit,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val pay by viewModel.payAsset.collectAsStateWithLifecycle()
    val receive by viewModel.receiveAsset.collectAsStateWithLifecycle()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    val selectedSlippage by viewModel.selectedSlippage.collectAsStateWithLifecycle()

    var isShowPriceImpactAlert by remember { mutableStateOf(false) }
    var isShowDetails by remember { mutableStateOf(false) }

    ObserveStartedState(viewModel::setRefreshEnabled)

    LaunchedEffect(select, selectedAssetId) {
        select ?: return@LaunchedEffect
        selectedAssetId?.let { viewModel.onSelect(select, it) }
        onSelectionConsumed()
    }

    SwapScene(
        viewState = viewState,
        pay = pay,
        receive = receive,
        payValue = viewModel.payValue,
        receiveValue = viewModel.receiveValue,
        showsSlippageIndicator = selectedSlippage != null,
        onAction = { action ->
            when (action) {
                is SwapAction.SelectAsset -> onSelect(action.type, pay?.asset?.id, receive?.asset?.id)

                is SwapAction.SelectPercent -> viewModel.onSelectPercent(action.percent)

                SwapAction.SwitchAssets -> viewModel.switchSwap()

                SwapAction.ShowDetails -> isShowDetails = true

                SwapAction.Slippage -> if (!viewState.isTransferLoading) {
                    viewModel.openSlippage()
                }

                SwapAction.Swap -> viewModel.onPrimaryAction(
                    onConfirm = onConfirm,
                    onShowPriceImpactWarning = { isShowPriceImpactAlert = true },
                    authorize = { action -> context.requestAuth(AuthRequest.Confirmation, action) },
                )

                SwapAction.Cancel -> onCancel()
            }
        },
    )

    PriceImpactWarningDialog(
        isVisible = isShowPriceImpactAlert,
        priceImpact = viewState.details?.summary?.priceImpactRow,
        asset = pay?.asset,
        onDismiss = { isShowPriceImpactAlert = false },
        onContinue = { context.requestAuth(AuthRequest.Confirmation) { viewModel.swap(onConfirm) } },
    )

    SwapDetailsBottomSheet(
        isVisible = isShowDetails,
        isLoading = viewState.isQuoteLoading && viewState.details == null,
        details = viewState.details,
        providers = viewState.providers,
        isProviderSelectable = viewState.allowsProviderSelection,
        onDismiss = { isShowDetails = false },
        expansion = SheetExpansion.Full,
        onProviderSelect = if (!viewState.isTransferLoading) viewModel::setProvider else null,
    )

    val slippage by viewModel.slippage.collectAsStateWithLifecycle()
    SwapSlippageBottomSheet(
        state = slippage,
        onAuto = viewModel::onSlippageAuto,
        onInput = viewModel::onSlippageInput,
        onDismiss = viewModel::closeSlippage,
    )
}
