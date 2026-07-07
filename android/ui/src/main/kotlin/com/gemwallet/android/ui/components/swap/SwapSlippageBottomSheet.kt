package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.swap.SwapSlippage
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SwapSlippageBottomSheet(
    isVisible: Boolean,
    currentBps: UInt?,
    warningThresholdBps: UInt,
    onSelect: (UInt?) -> Unit,
    onDismiss: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.swap_slippage),
    ) {
        var isAuto by remember(currentBps) { mutableStateOf(currentBps == null) }
        var selectedBps by remember(currentBps) { mutableStateOf(currentBps ?: SwapSlippage.defaultBps) }

        Column {
            SwitchProperty(
                text = stringResource(R.string.swap_slippage_auto),
                checked = isAuto,
                onCheckedChange = { checked ->
                    isAuto = checked
                    onSelect(if (checked) null else selectedBps)
                },
            )
            FooterText(
                text = stringResource(R.string.swap_slippage_auto_description),
                color = MaterialTheme.colorScheme.secondary,
            )

            if (!isAuto) {
                SingleChoiceSegmentedButtonRow(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = paddingDefault, vertical = paddingSmall),
                ) {
                    SwapSlippage.options.forEachIndexed { index, bps ->
                        SegmentedButton(
                            selected = selectedBps == bps,
                            onClick = {
                                selectedBps = bps
                                onSelect(bps)
                            },
                            shape = SegmentedButtonDefaults.itemShape(index = index, count = SwapSlippage.options.size),
                            colors = SegmentedButtonDefaults.colors(
                                activeContainerColor = MaterialTheme.colorScheme.primary.copy(alpha = alpha10),
                                activeContentColor = MaterialTheme.colorScheme.primary,
                            ),
                        ) {
                            Text(SwapSlippage.label(bps))
                        }
                    }
                }
                if (selectedBps >= warningThresholdBps) {
                    FooterText(
                        text = stringResource(R.string.swap_slippage_warning),
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
        }
    }
}

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
