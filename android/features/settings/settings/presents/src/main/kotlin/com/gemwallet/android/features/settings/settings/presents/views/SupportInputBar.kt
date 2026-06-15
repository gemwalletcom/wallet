package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.actionIconSize
import com.gemwallet.android.ui.theme.mainActionHeight
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall

private val messageInputCornerRadius = 24.dp

@Composable
internal fun SupportInputBar(onPickImage: () -> Unit, onSend: (String) -> Unit) {
    var input by remember { mutableStateOf("") }
    val canSend = input.isNotBlank()

    fun send() {
        val text = input.trim()
        if (text.isEmpty()) return
        onSend(text)
        input = ""
    }

    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = paddingDefault, vertical = paddingSmall),
        verticalAlignment = Alignment.Bottom,
        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        Surface(
            color = MaterialTheme.colorScheme.surfaceContainerHighest,
            shape = RoundedCornerShape(messageInputCornerRadius),
            modifier = Modifier.weight(1f),
        ) {
            Row(verticalAlignment = Alignment.Bottom) {
                IconButton(onClick = onPickImage) {
                    Icon(
                        imageVector = AppIcons.AddCircleOutlined,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary,
                    )
                }
                BasicTextField(
                    value = input,
                    onValueChange = { input = it },
                    modifier = Modifier.weight(1f).padding(end = paddingDefault),
                    textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    maxLines = 5,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                    keyboardActions = KeyboardActions(onSend = { send() }),
                    decorationBox = { inner ->
                        Box(
                            modifier = Modifier.heightIn(min = mainActionHeight),
                            contentAlignment = Alignment.CenterStart,
                        ) {
                            if (input.isEmpty()) {
                                Text(
                                    text = stringResource(R.string.support_message_placeholder),
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = MaterialTheme.colorScheme.secondary,
                                )
                            }
                            inner()
                        }
                    },
                )
            }
        }
        FilledIconButton(onClick = { send() }, enabled = canSend, modifier = Modifier.size(actionIconSize)) {
            Icon(imageVector = AppIcons.Send, contentDescription = null)
        }
    }
}
