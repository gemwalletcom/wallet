package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.SuffixTextField
import com.gemwallet.android.ui.components.SuggestionsBar
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.swap.SwapSlippage
import uniffi.gemstone.GemSlippageCheck
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SwapSlippageBottomSheet(
    isVisible: Boolean,
    currentBps: UInt?,
    defaultBps: UInt?,
    slippageCheck: (UInt) -> GemSlippageCheck,
    onConfirm: (UInt?) -> Unit,
    onDismiss: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.swap_slippage),
    ) {
        var isAuto by remember(currentBps) { mutableStateOf(currentBps == null) }
        var input by remember(currentBps) {
            mutableStateOf(currentBps?.let(SwapSlippage::format).orEmpty())
        }
        val focusRequester = remember { FocusRequester() }

        val bps = SwapSlippage.parseBps(input)
        val check = if (isAuto) null else bps?.let(slippageCheck)
        val isConfirmEnabled = isAuto || check == GemSlippageCheck.VALID || check == GemSlippageCheck.HIGH

        val commit by rememberUpdatedState {
            if (isConfirmEnabled) onConfirm(if (isAuto) null else bps)
        }
        DisposableEffect(Unit) {
            onDispose { commit() }
        }

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .height(SlippageSheetHeight),
        ) {
            SwitchProperty(
                text = stringResource(R.string.swap_slippage_auto),
                checked = isAuto,
                onCheckedChange = { isAuto = it },
            )
            FooterText(
                text = stringResource(R.string.swap_slippage_auto_description),
                color = MaterialTheme.colorScheme.secondary,
            )

            if (!isAuto) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .listItem(ListPosition.Single)
                        .padding(paddingDefault),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    PropertyTitleText(
                        stringResource(R.string.swap_slippage),
                        info = InfoSheetEntity.Slippage,
                    )
                    SuffixTextField(
                        modifier = Modifier
                            .weight(1f)
                            .padding(start = paddingSmall),
                        value = input,
                        placeholder = defaultBps?.let(SwapSlippage::format).orEmpty(),
                        onValueChange = { input = SwapSlippage.sanitize(it) },
                        suffix = "%",
                        focusRequester = focusRequester,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    )
                }
                when (check) {
                    GemSlippageCheck.ABOVE_MAXIMUM -> FooterText(
                        text = stringResource(R.string.common_maximum_value, SwapSlippage.maxPercentLabel),
                        color = MaterialTheme.colorScheme.error,
                    )
                    GemSlippageCheck.BELOW_MINIMUM -> FooterText(
                        text = stringResource(R.string.common_minimum_value, SwapSlippage.minPercentLabel),
                        color = MaterialTheme.colorScheme.error,
                    )
                    GemSlippageCheck.HIGH -> FooterText(
                        text = stringResource(R.string.swap_slippage_warning),
                        color = MaterialTheme.colorScheme.error,
                    )
                    GemSlippageCheck.VALID, null -> Unit
                }
                Spacer(modifier = Modifier.weight(1f))
                SuggestionsBar(
                    labels = SwapSlippage.suggestionsBps.map { "${SwapSlippage.format(it)}%" },
                    modifier = Modifier.padding(horizontal = paddingDefault, vertical = paddingSmall),
                    onSelected = { index -> input = SwapSlippage.format(SwapSlippage.suggestionsBps[index]) },
                )
            }
        }

        LaunchedEffect(isAuto) {
            if (!isAuto) {
                try {
                    focusRequester.requestFocus()
                } catch (_: Throwable) {}
            }
        }
    }
}

private val SlippageSheetHeight = 296.dp

@Composable
private fun FooterText(text: String, color: Color) {
    Text(
        modifier = Modifier
            .fillMaxWidth()
            .padding(
                horizontal = adaptivePadding(default = paddingDefault, compact = paddingSmall) + paddingMiddle,
                vertical = paddingSmall,
            ),
        text = text,
        color = color,
        style = MaterialTheme.typography.bodySmall,
    )
}
