package com.gemwallet.android.features.swap.views

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.SwapVert
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.models.SwapState
import com.gemwallet.android.features.swap.views.components.SwapAction
import com.gemwallet.android.features.swap.views.components.SwapError
import com.gemwallet.android.features.swap.views.components.SwapItem
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.theme.iconSize

@Composable
internal fun SwapScene(
    swapState: SwapState,
    pay: AssetInfo?,
    receive: AssetInfo?,
    payEquivalent: String,
    receiveEquivalent: String,
    swapDetails: SwapDetailsUIModel?,
    onShowPriceImpactWarning: () -> Unit,
    onSelectAsset: (SwapItemType) -> Unit,
    payValue: TextFieldState,
    receiveValue: TextFieldState,
    switchSwap: () -> Unit,
    onDetails: () -> Unit,
    onCancel: () -> Unit,
    onSwap: () -> Unit,
) {
    val keyboardController = LocalSoftwareKeyboardController.current

    Scene(
        title = stringResource(id = R.string.wallet_swap),
        mainAction = {
            SwapAction(swapState) {
                when {
                    swapState is SwapState.Error && swapState.error is SwapError.InputAmountTooSmall -> {
                        val value = pay?.asset?.let {
                            (swapState.error as SwapError.InputAmountTooSmall).getValue(it)
                        } ?: return@SwapAction
                        payValue.clearText()
                        payValue.setTextAndPlaceCursorAtEnd(value.toString())
                    }
                    swapDetails?.shouldShowPriceImpactWarning == true -> onShowPriceImpactWarning()
                    else -> onSwap()
                }
            }
        },
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                SwapSectionHeader(R.string.swap_you_pay)
            }
            item {
                SwapItem(
                    type = SwapItemType.Pay,
                    item = pay,
                    equivalent = payEquivalent,
                    state = payValue,
                    onAssetSelect = {
                        keyboardController?.hide()
                        onSelectAsset(SwapItemType.Pay)
                    }
                )
            }
            item {
                Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                    Box(
                        modifier = Modifier
                            .size(iconSize)
                            .clickable(onClick = switchSwap),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            imageVector = Icons.Default.SwapVert,
                            contentDescription = stringResource(R.string.wallet_swap),
                        )
                    }
                }
            }
            item {
                SwapSectionHeader(R.string.swap_you_receive)
            }
            item {
                SwapItem(
                    type = SwapItemType.Receive,
                    item = receive,
                    equivalent = receiveEquivalent,
                    state = receiveValue,
                    calculating = swapState == SwapState.GetQuote,
                    onAssetSelect = {
                        keyboardController?.hide()
                        onSelectAsset(SwapItemType.Receive)
                    }

                )
            }
            item {
                swapDetails?.let {
                    SwapDetailsSummaryItem(model = it, onClick = onDetails)
                }
            }

            item {
                SwapError(swapState, pay)
            }
        }
    }
}

@Composable
private fun SwapSectionHeader(resId: Int) {
    Text(
        modifier = Modifier
            .sectionHeaderItem(),
        text = stringResource(resId),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.secondary,
    )
}
