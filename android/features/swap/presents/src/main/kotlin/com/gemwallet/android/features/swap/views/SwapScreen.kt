package com.gemwallet.android.features.swap.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.features.swap.viewmodels.SwapViewModel
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.views.dialogs.PriceImpactWarningDialog
import com.gemwallet.android.ui.ObserveStartedState
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.wallet.core.primitives.AssetId

@Composable
fun SwapScreen(
    payId: AssetId?,
    receiveId: AssetId?,
    select: SwapItemType?,
    viewModel: SwapViewModel = hiltViewModel(),
    onSelectionConsumed: () -> Unit,
    onSelect: (select: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
    onCancel: () -> Unit,
) {
    val pay by viewModel.payAsset.collectAsStateWithLifecycle()
    val receive by viewModel.receiveAsset.collectAsStateWithLifecycle()
    val fromEquivalent by viewModel.payEquivalentFormatted.collectAsStateWithLifecycle()
    val toEquivalent by viewModel.toEquivalentFormatted.collectAsStateWithLifecycle()
    val swapState by viewModel.uiState.collectAsStateWithLifecycle()
    val swapDetails by viewModel.swapDetails.collectAsStateWithLifecycle()

    var isShowPriceImpactAlert by remember { mutableStateOf(false) }
    var isShowDetails by remember { mutableStateOf(false) }

    ObserveStartedState(viewModel::setRefreshEnabled)

    LaunchedEffect(payId, receiveId, select) {
        select ?: return@LaunchedEffect
        val assetId = when (select) {
            SwapItemType.Pay -> payId
            SwapItemType.Receive -> receiveId
        } ?: run {
            onSelectionConsumed()
            return@LaunchedEffect
        }

        viewModel.onSelect(select, assetId)
        onSelectionConsumed()
    }

    val onPrimaryAction: () -> Unit = {
        viewModel.onPrimaryAction(
            onConfirm = onConfirm,
            onShowPriceImpactWarning = { isShowPriceImpactAlert = true },
        )
    }

    SwapScene(
        swapState = swapState,
        pay = pay,
        receive = receive,
        swapDetails = swapDetails,
        payEquivalent = fromEquivalent,
        receiveEquivalent = toEquivalent,
        onSelectAsset = { type ->
            onSelect(type, pay?.id(), receive?.id())
        },
        switchSwap = viewModel::switchSwap,
        payValue = viewModel.payValue,
        receiveValue = viewModel.receiveValue,
        onCancel = onCancel,
        onDetails = { isShowDetails = true },
        onPrimaryAction = onPrimaryAction,
        onSelectPayPercent = viewModel::onSelectPayPercent,
    )

    PriceImpactWarningDialog(
        isVisible = isShowPriceImpactAlert,
        priceImpact = swapDetails?.priceImpact,
        asset = pay?.asset,
        onDismiss = { isShowPriceImpactAlert = false },
        onContinue = { viewModel.swap(onConfirm) },
    )

    SwapDetailsBottomSheet(
        isVisible = isShowDetails,
        isLoading = swapState.isQuoteLoading && swapDetails == null,
        model = swapDetails,
        onDismiss = { isShowDetails = false },
        skipPartiallyExpanded = true,
        onProviderSelect = if (swapState.isQuoteInteractionEnabled) viewModel::setProvider else null,
    )
}
