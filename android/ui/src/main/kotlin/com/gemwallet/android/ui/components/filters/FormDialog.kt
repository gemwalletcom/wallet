package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.theme.normalPadding

@Composable
fun FormDialog(
    isVisible: Boolean,
    title: String,
    expansion: SheetExpansion = SheetExpansion.Partial,
    onDismiss: () -> Unit,
    onClear: (() -> Unit)? = null,
    doneAction: @Composable (() -> Unit)? = null,
    bottomAction: @Composable (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        expansion = expansion,
        onDismissRequest = onDismiss,
        title = title,
        actions = {
            if (onClear != null) {
                TextButton(onClick = onClear) {
                    Text(stringResource(R.string.filter_clear))
                }
            }
            doneAction?.invoke()
        },
    ) {
        Column {
            if (bottomAction == null) {
                content()
            } else {
                Column(modifier = Modifier.weight(1f)) {
                    content()
                }
                Box(modifier = Modifier.normalPadding()) { bottomAction() }
            }
        }
    }
}
