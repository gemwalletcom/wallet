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
import com.gemwallet.android.features.swap.viewmodels.models.SwapState
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
    onSelect: (select: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
    onCancel: () -> Unit,
) {
    val pay by viewModel.payAsset.collectAsStateWithLifecycle()
    val receive by viewModel.receiveAsset.collectAsStateWithLifecycle()
    val fromEquivalent by viewModel.payEquivalentFormatted.collectAsStateWithLifecycle()
    val toEquivalent by viewModel.toEquivalentFormatted.collectAsStateWithLifecycle()
    val swapState by viewModel.uiSwapScreenState.collectAsStateWithLifecycle()
    val swapDetails by viewModel.swapDetails.collectAsStateWithLifecycle()

    var isShowPriceImpactAlert by remember { mutableStateOf(false) }
    var isShowDetails by remember { mutableStateOf(false) }

    ObserveStartedState(viewModel::setRefreshEnabled)

    LaunchedEffect(payId, receiveId, select) {
        select ?: return@LaunchedEffect
        viewModel.onSelect(
            select,
            when (select) {
                SwapItemType.Pay -> payId ?: return@LaunchedEffect
                SwapItemType.Receive -> receiveId ?: return@LaunchedEffect
            }
        )
    }

    val onSwap: () -> Unit = {
        when (swapState) {
            SwapState.Ready -> viewModel.swap(onConfirm)
            is SwapState.Error -> viewModel.refresh()
            else -> {}
        }
    }

    SwapScene(
        swapState = swapState,
        pay = pay,
        receive = receive,
        swapDetails = swapDetails,
        payEquivalent = fromEquivalent,
        receiveEquivalent = toEquivalent,
        onShowPriceImpactWarning = { isShowPriceImpactAlert = true },
        onSelectAsset = { type ->
            onSelect(type, pay?.id(), receive?.id())
        },
        switchSwap = viewModel::switchSwap,
        payValue = viewModel.payValue,
        receiveValue = viewModel.receiveValue,
        onCancel = onCancel,
        onDetails = { isShowDetails = true },
        onSwap = onSwap,
    )

    PriceImpactWarningDialog(
        isVisible = isShowPriceImpactAlert,
        priceImpact = swapDetails?.priceImpact,
        asset = pay?.asset,
        onDismiss = { isShowPriceImpactAlert = false },
        onContinue = onSwap,
    )

    SwapDetailsBottomSheet(
        isVisible = isShowDetails,
        isLoading = swapState == SwapState.GetQuote,
        model = swapDetails,
        onDismiss = { isShowDetails = false },
        skipPartiallyExpanded = true,
        onProviderSelect = { provider ->
            viewModel.setProvider(provider)
        },
    )
}
