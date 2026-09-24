package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.SuffixTextField
import com.gemwallet.android.ui.components.SuggestionsBar
import com.gemwallet.android.ui.components.fields.decimalKeyboardOptions
import com.gemwallet.android.ui.components.fields.requestFocusIfAttached
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SwapSlippageBottomSheet(state: SlippageStateUIModel?, onAuto: (Boolean) -> Unit, onInput: (String) -> Unit, onDismiss: () -> Unit) {
    ModalBottomSheet(
        isVisible = state != null,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = stringResource(R.string.swap_slippage),
    ) {
        state ?: return@ModalBottomSheet
        val focusRequester = remember { FocusRequester() }

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .height(SlippageSheetHeight),
        ) {
            SwitchProperty(
                text = stringResource(R.string.swap_slippage_auto),
                checked = state.isAuto,
                onCheckedChange = onAuto,
            )
            FooterText(
                text = stringResource(R.string.swap_slippage_auto_description),
                color = MaterialTheme.colorScheme.secondary,
            )

            if (!state.isAuto) {
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
                        value = state.input,
                        placeholder = state.placeholder,
                        onValueChange = onInput,
                        suffix = "%",
                        focusRequester = focusRequester,
                        keyboardOptions = decimalKeyboardOptions(),
                    )
                }
                state.footerText?.let { FooterText(text = it, color = MaterialTheme.colorScheme.error) }
                Spacer(modifier = Modifier.weight(1f))
                SuggestionsBar(
                    labels = state.suggestions.map { it.label },
                    modifier = Modifier.padding(horizontal = paddingDefault, vertical = paddingSmall),
                    onSelected = { index -> onInput(state.suggestions[index].input) },
                )
            }
        }

        LaunchedEffect(state.isAuto) {
            if (!state.isAuto) {
                focusRequester.requestFocusIfAttached()
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
