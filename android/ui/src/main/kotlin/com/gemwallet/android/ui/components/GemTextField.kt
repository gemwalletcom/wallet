package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.KeyboardActionHandler
import androidx.compose.foundation.text.input.TextFieldDecorator
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.paddingDefault

private val minInputHeight = 40.dp

@Composable
fun GemTextField(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    label: String = "",
    error: String = "",
    trailing: (@Composable () -> Unit)? = null,
    singleLine: Boolean = true,
    readOnly: Boolean = false,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    onKeyboardAction: KeyboardActionHandler? = null,
    listPosition: ListPosition = ListPosition.Single,
    errorDivider: Boolean = false,
) {
    val hasFloatingLabel = value.isNotEmpty() && label.isNotEmpty()
    val state = rememberValueTextFieldState(value)

    Column(
        modifier = modifier
            .fillMaxWidth()
            .listItem(listPosition)
            .padding(paddingDefault),
        verticalArrangement = Arrangement.Center,
    ) {
        Row(
            modifier = Modifier.heightIn(min = minInputHeight),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicTextField(
                modifier = Modifier.weight(1f),
                state = state,
                inputTransformation = valueChangeTransformation(value, onValueChange),
                readOnly = readOnly,
                lineLimits = if (singleLine) TextFieldLineLimits.SingleLine else TextFieldLineLimits.MultiLine(),
                textStyle = MaterialTheme.typography.bodyLarge.copy(
                    color = MaterialTheme.colorScheme.onSurface,
                ),
                keyboardOptions = keyboardOptions,
                onKeyboardAction = onKeyboardAction,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                decorator = TextFieldDecorator { innerTextField ->
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.Center,
                    ) {
                        if (hasFloatingLabel) {
                            Text(
                                text = label,
                                color = MaterialTheme.colorScheme.secondary,
                                style = MaterialTheme.typography.bodySmall,
                            )
                        }
                        Box(contentAlignment = Alignment.CenterStart) {
                            if (label.isNotEmpty() && !hasFloatingLabel) {
                                Text(
                                    text = label,
                                    color = MaterialTheme.colorScheme.secondary,
                                    style = MaterialTheme.typography.bodyLarge,
                                )
                            }
                            innerTextField()
                        }
                    }
                },
            )
            trailing?.invoke()
        }
        if (error.isNotEmpty()) {
            if (errorDivider) {
                Spacer4()
                HorizontalDivider()
            }
            Spacer4()
            Text(
                modifier = Modifier,
                text = error,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}
