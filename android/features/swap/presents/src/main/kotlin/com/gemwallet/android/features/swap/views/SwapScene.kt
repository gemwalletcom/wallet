package com.gemwallet.android.features.swap.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.SwapVert
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.features.swap.views.components.SwapAction
import com.gemwallet.android.features.swap.views.components.SwapError
import com.gemwallet.android.features.swap.views.components.SwapItem
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.space0

@Composable
internal fun SwapScene(
    swapState: SwapUiState,
    pay: AssetInfo?,
    receive: AssetInfo?,
    payEquivalent: String,
    receiveEquivalent: String,
    swapDetails: SwapDetailsUIModel?,
    onSelectAsset: (SwapItemType) -> Unit,
    payValue: TextFieldState,
    receiveValue: TextFieldState,
    switchSwap: () -> Unit,
    onDetails: () -> Unit,
    onCancel: () -> Unit,
    onPrimaryAction: () -> Unit,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    val imeVisible = WindowInsets.ime.getBottom(LocalDensity.current) > 0
    var pendingDetails by remember { mutableStateOf(false) }

    LaunchedEffect(imeVisible) {
        if (!imeVisible && pendingDetails) {
            pendingDetails = false
            onDetails()
        }
    }

    Scene(
        title = stringResource(id = R.string.wallet_swap),
        mainAction = {
            SwapAction(swapState, onPrimaryAction)
        },
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                SwapSectionHeader(R.string.swap_you_pay)
            }
            item {
                SwapItem(
                    item = pay,
                    equivalent = payEquivalent,
                    state = payValue,
                    interaction = swapState.payItemInteraction,
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
                            .clip(MaterialTheme.shapes.medium)
                            .clickable(
                                enabled = swapState.isQuoteInteractionEnabled,
                                onClick = switchSwap,
                            ),
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
                SwapSectionHeader(R.string.swap_you_receive, topPadding = space0)
            }
            item {
                SwapItem(
                    item = receive,
                    equivalent = receiveEquivalent,
                    state = receiveValue,
                    calculating = swapState.isReceiveLoading,
                    interaction = swapState.receiveItemInteraction,
                    onAssetSelect = {
                        keyboardController?.hide()
                        onSelectAsset(SwapItemType.Receive)
                    }

                )
            }
            item {
                swapDetails?.let {
                    SwapDetailsSummaryItem(model = it, onClick = {
                        if (imeVisible) {
                            keyboardController?.hide()
                            pendingDetails = true
                        } else onDetails()
                    })
                }
            }

            item {
                SwapError(swapState, pay)
            }
        }
    }
}

@Composable
private fun SwapSectionHeader(resId: Int, topPadding: Dp? = null) {
    Text(
        modifier = Modifier
            .sectionHeaderItem(paddingVertical = topPadding),
        text = stringResource(resId),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.secondary,
    )
}
