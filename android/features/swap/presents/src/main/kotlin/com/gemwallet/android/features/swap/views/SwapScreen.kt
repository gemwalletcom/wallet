package com.gemwallet.android.features.swap.views

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
import com.gemwallet.android.features.swap.viewmodels.SwapViewModel
import com.gemwallet.android.features.swap.views.dialogs.PriceImpactWarningDialog
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
    val swapState by viewModel.uiState.collectAsStateWithLifecycle()
    val swapDetails by viewModel.swapDetails.collectAsStateWithLifecycle()
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
        swapState = swapState,
        pay = pay,
        receive = receive,
        swapDetails = swapDetails,
        payEquivalent = swapState.payEquivalent,
        receiveEquivalent = swapState.receiveEquivalent,
        payBalance = swapState.payBalance,
        receiveBalance = swapState.receiveBalance,
        payValue = viewModel.payValue,
        receiveValue = viewModel.receiveValue,
        showsSlippageIndicator = selectedSlippage != null,
        onAction = { action ->
            when (action) {
                is SwapSceneAction.SelectAsset -> onSelect(action.type, pay?.id(), receive?.id())

                is SwapSceneAction.SelectPercent -> viewModel.onSelectPercent(action.percent)

                SwapSceneAction.SwitchAssets -> viewModel.switchSwap()

                SwapSceneAction.ShowDetails -> isShowDetails = true

                SwapSceneAction.Slippage -> if (swapState.isQuoteInteractionEnabled) {
                    viewModel.openSlippage()
                }

                SwapSceneAction.Swap -> viewModel.onPrimaryAction(
                    onConfirm = onConfirm,
                    onShowPriceImpactWarning = { isShowPriceImpactAlert = true },
                    authorize = { action -> context.requestAuth(AuthRequest.Confirmation, action) },
                )

                SwapSceneAction.Cancel -> onCancel()
            }
        },
    )

    PriceImpactWarningDialog(
        isVisible = isShowPriceImpactAlert,
        priceImpact = swapDetails?.priceImpact,
        asset = pay?.asset,
        onDismiss = { isShowPriceImpactAlert = false },
        onContinue = { context.requestAuth(AuthRequest.Confirmation) { viewModel.swap(onConfirm) } },
    )

    SwapDetailsBottomSheet(
        isVisible = isShowDetails,
        isLoading = swapState.isQuoteLoading && swapDetails == null,
        model = swapDetails,
        onDismiss = { isShowDetails = false },
        expansion = SheetExpansion.Full,
        onProviderSelect = if (swapState.isQuoteInteractionEnabled) viewModel::setProvider else null,
    )

    val slippage by viewModel.slippage.collectAsStateWithLifecycle()
    SwapSlippageBottomSheet(
        state = slippage,
        onAuto = viewModel::onSlippageAuto,
        onInput = viewModel::onSlippageInput,
        onDismiss = viewModel::closeSlippage,
    )
}
